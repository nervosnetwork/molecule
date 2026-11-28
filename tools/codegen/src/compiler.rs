use std::{env, io, io::Write as _, path};

#[cfg(feature = "compiler-plugin")]
use std::process;

use crate::{generator, parser};

#[cfg(feature = "compiler-plugin")]
use crate::ir;

pub struct Compiler {
    target: Option<generator::Target>,
    input: Option<Input>,
    output: Option<Output>,
}

pub(crate) enum Input {
    SchemaFile(path::PathBuf),
    #[cfg(feature = "compiler-plugin")]
    Intermediate(ir::Format, Vec<u8>),
}

pub(crate) enum Output {
    Directory(path::PathBuf),
    Stdout,
    #[cfg(feature = "compiler-plugin")]
    PluginProcess(process::Child),
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            target: None,
            input: None,
            output: Some(Output::Stdout),
        }
    }

    pub fn generate_code(&mut self, lang: generator::Language) -> &mut Self {
        self.target.replace(generator::Target::Language(lang));
        self
    }

    #[cfg(feature = "compiler-plugin")]
    pub fn generate_intermediate(&mut self, format: ir::Format) -> &mut Self {
        self.target.replace(generator::Target::Intermediate(format));
        self
    }

    pub fn input_schema_file<P: AsRef<path::Path>>(&mut self, path: P) -> &mut Self {
        self.input
            .replace(Input::SchemaFile(path.as_ref().to_path_buf()));
        self
    }

    #[cfg(feature = "compiler-plugin")]
    pub fn input_intermediate(&mut self, format: ir::Format, data: Vec<u8>) -> &mut Self {
        self.input.replace(Input::Intermediate(format, data));
        self
    }

    pub fn output_dir_set_default(&mut self) -> &mut Self {
        let out_dir = path::PathBuf::from(&env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
        self.output_dir(out_dir)
    }

    pub fn output_dir<P: AsRef<path::Path>>(&mut self, path: P) -> &mut Self {
        self.output
            .replace(Output::Directory(path.as_ref().to_path_buf()));
        self
    }

    #[cfg(feature = "compiler-plugin")]
    pub fn output_plugin_process(&mut self, child: process::Child) -> &mut Self {
        self.output.replace(Output::PluginProcess(child));
        self
    }

    pub fn run(&mut self) -> Result<(), String> {
        let Self {
            target,
            ref input,
            #[cfg(not(feature = "compiler-plugin"))]
            ref output,
            #[cfg(feature = "compiler-plugin")]
            ref mut output,
        } = self;
        let target = target.ok_or("target is not set: generate code or intermediate data")?;
        let input = input
            .as_ref()
            .ok_or("input is not set: schema file or intermediate data")?;

        #[cfg(not(feature = "compiler-plugin"))]
        let output = output.as_ref().ok_or("output is not set")?;
        #[cfg(feature = "compiler-plugin")]
        let output = output.as_mut().ok_or("output is not set")?;

        let ast = match input {
            Input::SchemaFile(ref file_path) => parser::Parser::parse(file_path),
            #[cfg(feature = "compiler-plugin")]
            Input::Intermediate(format, ref data) => format.recover(data)?,
        };
        let namespace = ast.namespace().to_owned();
        let generator = generator::Generator::new(ast);

        let mut output_data = Vec::<u8>::new();
        generator
            .generate(target, &mut output_data)
            .map_err(|err| format!("failed to write data by generator: {}", err))?;

        match output {
            Output::Directory(ref out_dir) => {
                if !crate::utils::valid_namespace(&namespace) {
                    return Err(format!("E_OUTPUT: invalid output namespace {namespace:?}"));
                }
                let out_file = out_dir.join(format!("{namespace}.{}", target.extension()));
                crate::output::write_atomic(&out_file, &output_data)
                    .map_err(|err| format!("E_OUTPUT: {}: {err}", out_file.display()))?;
            }
            Output::Stdout => {
                let stdout = io::stdout();
                let mut stdout_handle = stdout.lock();
                stdout_handle
                    .write_all(&output_data)
                    .and_then(|_| stdout_handle.flush())
                    .map_err(|err| format!("E_OUTPUT: {err}"))?;
            }
            #[cfg(feature = "compiler-plugin")]
            Output::PluginProcess(ref mut process) => {
                {
                    let child_stdin = process.stdin.as_mut().unwrap();
                    child_stdin.write_all(&output_data).unwrap();
                    child_stdin.flush().unwrap();
                }
                if let Ok(status) = process.wait() {
                    if !status.success() {
                        process::exit(1)
                    }
                } else {
                    eprintln!("Error: failed to execute the plugin");
                    process::exit(1)
                }
            }
        }

        Ok(())
    }
}
