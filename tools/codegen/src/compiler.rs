use std::{
    env, fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
};

use crate::{Generator, Language};

pub enum Output {
    Stdout,
    Directory(PathBuf),
}

pub struct Compiler {
    language: Option<Language>,
    file_path: Option<PathBuf>,
    output: Output,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            language: None,
            file_path: None,
            output: Output::Stdout,
        }
    }

    pub fn language(&mut self, lang: Language) -> &mut Self {
        self.language.replace(lang);
        self
    }

    pub fn file_path<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.file_path.replace(path.as_ref().to_path_buf());
        self
    }

    pub fn default_out_dir(&mut self) -> &mut Self {
        let out_dir = PathBuf::from(&env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
        self.output = Output::Directory(out_dir);
        self
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.output = Output::Directory(path.as_ref().to_path_buf());
        self
    }

    pub fn run(&mut self) {
        let lang = self.language.unwrap();

        let generator = Generator::new(&self.file_path.as_ref().unwrap());

        match self.output {
            Output::Directory(ref out_dir) => {
                let file_name = self
                    .file_path
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_owned();

                let mut out_file = out_dir.to_owned();
                out_file.push(file_name);
                out_file.set_extension(lang.extension());

                let mut file_out = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&out_file)
                    .unwrap();

                generator.generate(lang, &mut file_out).unwrap();
                file_out.flush().unwrap();
            }
            Output::Stdout => {
                let stdout = io::stdout();
                let mut stdout_handle = stdout.lock();
                generator.generate(lang, &mut stdout_handle).unwrap();
                stdout_handle.flush().unwrap();
            }
        }
    }
}
