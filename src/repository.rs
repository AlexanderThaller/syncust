use chunker::Chunker;
use failure::{
    Error,
    ResultExt,
};
use repofile::RepoFile;
use serde_json::to_writer;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs::{
    create_dir_all,
    rename,
    File,
};
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Fail)]
enum RepositoryError {
    #[fail(display = "repository is already initialized")] AlreadyInitialized,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    path: PathBuf,
    files: BTreeMap<PathBuf, RepoFile>,
    sublayers: usize,
}

impl Default for Repository {
    fn default() -> Repository {
        Repository {
            path: PathBuf::new(),
            files: BTreeMap::new(),
            sublayers: 4,
        }
    }
}

impl Repository {
    pub fn with_path(self, path: PathBuf) -> Repository {
        Repository { path: path, ..self }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        if self.get_data_path().exists() {
            Err(RepositoryError::AlreadyInitialized)?
        }

        for entry in WalkDir::new(&self.path) {
            trace!("repository::Repository::init: entry - {:?}", entry);

            let entry = entry?;
            let path = entry.path();
            let file = RepoFile::from_path(&path)?;

            self.files.insert(path.to_path_buf(), file);
        }

        create_dir_all(self.get_data_path()).context(format_err!("can not create data dir"))?;

        for (path, file) in &self.files {
            self.move_file_to_object_store(path, file)?;
        }

        self.write_repodata()?;

        Ok(())
    }

    fn write_repodata(&self) -> Result<(), Error> {
        let data_path = self.get_data_path().join("data.json");
        let datafile = File::create(&data_path).context(format_err!("can not create datafile {:?}", data_path))?;
        to_writer(datafile, self).context(format_err!("can not serialize repository data to datafile"))?;

        Ok(())
    }

    fn move_file_to_object_store<P: AsRef<Path> + Debug>(&self, path: P, file: &RepoFile) -> Result<(), Error> {
        let mut objects_path = self.get_objects_path();
        if file.is_dir {
            return Ok(());
        }

        let hash = file.hash.clone().ok_or(format_err!(
            "file {:?} does not have a a hash. this should never happen",
            path
        ))?;

        let mut chunker = Chunker::new(hash.as_str(), 2);

        for _sublayers in { 0..self.sublayers } {
            objects_path = objects_path.join(chunker.next().ok_or(format_err!(
                "chunker for path {:?} has no avaible chunks",
                path
            ))?);

            create_dir_all(&objects_path)?;
        }

        objects_path = objects_path.join(hash);

        debug!(
            "repository::Repository::move_file_to_object_store: path - {:?}, file - {:?}, objects_path - {:?}",
            path, file, objects_path
        );

        rename(&path, &objects_path).context(format_err!(
            "can not move file from {:?} to {:?}",
            path,
            objects_path
        ))?;

        symlink(&objects_path, &path).context(format_err!(
            "can not create symlink {:?} -> {:?}",
            path,
            objects_path
        ))?;

        Ok(())
    }

    fn get_data_path(&self) -> PathBuf {
        self.path.clone().join(".syncust")
    }

    fn get_objects_path(&self) -> PathBuf {
        self.get_data_path().join("objects")
    }
}
