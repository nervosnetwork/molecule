use std::{
    env, fs,
    io::Read as _,
    path::{Path, PathBuf},
};

use crate::Generator;

pub struct Compiler {
    file_path: Option<PathBuf>,
    out_dir: PathBuf,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            file_path: None,
            out_dir: PathBuf::from(&env::var("OUT_DIR").unwrap()),
        }
    }

    pub fn file_path<P>(&mut self, path: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.file_path.replace(path.as_ref().to_path_buf());
        self
    }

    pub fn out_dir<P>(&mut self, path: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.out_dir = path.as_ref().to_path_buf();
        self
    }

    pub fn run(&mut self) {
        let file_name = self
            .file_path
            .as_ref()
            .unwrap()
            .file_name()
            .unwrap()
            .to_owned();
        let mut out_file = self.out_dir.clone();
        out_file.push(file_name);
        out_file.set_extension("rs");

        let mut file_in = fs::OpenOptions::new()
            .read(true)
            .open(&self.file_path.as_ref().unwrap())
            .unwrap();
        let mut buffer = String::new();
        file_in.read_to_string(&mut buffer).unwrap();

        let mut file_out = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&out_file)
            .unwrap();

        let generator = Generator::new(&buffer);
        generator.generate_rust(&mut file_out).unwrap();
    }
}
