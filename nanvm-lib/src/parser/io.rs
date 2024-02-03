use std::{fs, io::Error};

pub trait Io {
    fn read_root(&self) -> Result<String, Error>;
    fn read(&self, path: &str) -> Result<String, Error>;
}

pub struct FileSystem<'a> {
    pub root_path: &'a str,
}

impl Io for FileSystem<'_> {
    fn read_root(&self) -> Result<String, Error> {
        fs::read_to_string(self.root_path)
    }

    fn read(&self, path: &str) -> Result<String, Error> {
        let full_path = format!("{}{}", self.root_path, path);
        fs::read_to_string(full_path)
    }
}
