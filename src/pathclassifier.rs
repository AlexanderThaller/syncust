use failure::Error;
use std::fmt::Debug;
use std::path::Path;

pub enum PathType {
    Local,
}

// TODO: Implement ssh path type detection
pub fn from_path<P: AsRef<Path> + Debug>(_path: P) -> Result<PathType, Error> {
    Ok(PathType::Local)
}
