use crate::reader::reader::Reader;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct FileReader {
	path: PathBuf,
}

impl FileReader {
	pub fn new(path: PathBuf) -> Self {
		FileReader { path: path }
	}
}

impl Reader for FileReader {
	fn load(&self) -> Result<String> {
		let text = fs::read_to_string(self.path.clone())?;
		Ok(text)
	}
}
