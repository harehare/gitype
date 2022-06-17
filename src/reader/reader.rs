use anyhow::Result;

pub trait Reader {
	fn load(&self) -> Result<String>;
}
