use failure::{
    Error,
    ResultExt,
};
use sha2::{
    Digest,
    Sha256,
};
use std::fmt::Debug;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RepoFile {
    pub hash: Option<String>,
    pub is_dir: bool,
    pub permissions: u32,
}

impl RepoFile {
    pub fn from_path<P: AsRef<Path> + Debug>(path: P) -> Result<RepoFile, Error> {
        trace!("repofile::from_path: path- {:?}", path);

        let mut file = File::open(&path).context(format_err!("can not open path {:?}", path))?;
        trace!("repofile::from_path: file - {:?}", file);

        let metadata = file.metadata()
            .context(format_err!("can not get is_dir for file {:?}", path))?;

        trace!("repofile::from_path: metadata - {:?}", metadata);

        let is_dir = metadata.is_dir();

        let hash = if is_dir {
            None
        } else {
            Some(format!("{:x}", Sha256::digest_reader(&mut file)?))
        };

        Ok(RepoFile {
            is_dir: is_dir,
            permissions: metadata.permissions().mode(),
            hash: hash,
        })
    }
}
