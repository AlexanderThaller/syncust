use failure::{
    Error,
    ResultExt,
};
use sha2::{
    Digest,
    Sha256,
};
use std::fmt::Debug;
use std::fs::{
    symlink_metadata,
    File,
};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoFile {
    pub hash: Option<String>,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub len: u64,
    pub modified: SystemTime,
    pub permissions: u32,
}

impl RepoFile {
    pub fn from_path<P: AsRef<Path> + Debug>(path: P) -> Result<RepoFile, Error> {
        trace!("repofile::from_path: path- {:?}", path);

        let mut file = File::open(&path).context(format_err!("can not open path {:?}", path))?;
        trace!("repofile::from_path: file - {:?}", file);

        // NOTE: We dont want to follow symlinks as we want to replicate the symlinks
        // in other repositories.
        let metadata = symlink_metadata(&path).context(format_err!("can not get metadata for file {:?}", path))?;

        trace!("repofile::from_path: metadata - {:?}", metadata);

        let is_dir = metadata.is_dir();

        let hash = if is_dir {
            None
        } else {
            Some(format!("{:x}", Sha256::digest_reader(&mut file)?))
        };

        Ok(RepoFile {
            hash: hash,
            is_dir: is_dir,
            is_symlink: metadata.file_type().is_symlink(),
            len: metadata.len(),
            modified: metadata
                .modified()
                .context(format_err!("can not get modified time for file {:?}", path))?,
            permissions: metadata.permissions().mode(),
        })
    }
}
