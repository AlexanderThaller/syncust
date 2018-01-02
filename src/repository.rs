use chunker::Chunker;
use failure::{
    Error,
    ResultExt,
};
use rayon::prelude::*;
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

        {
            let paths: Vec<_> = WalkDir::new(&self.path)
                .into_iter()
                .map(|entry| entry.unwrap())
                .map(|entry| entry.path().to_path_buf())
                .collect();

            let entries: Vec<_> = paths
                .par_iter()
                .map(|path| (RepoFile::from_path(&path), path))
                .collect();

            for entry in entries {
                trace!("repository::Repository::init: entry - {:?}", entry);
                let file = entry.0?;

                self.files.insert(entry.1.to_path_buf(), file);
            }
        }

        return Ok(());

        create_dir_all(self.get_data_path()).context("can not create data dir")?;

        for (path, file) in &self.files {
            self.move_file_to_object_store(path, file)?;
        }

        self.write_repodata()?;

        Ok(())
    }

    fn write_repodata(&self) -> Result<(), Error> {
        let data_path = self.get_data_path().join("data.json");
        let datafile = File::create(&data_path).context(format_err!("can not create datafile {:?}", data_path))?;
        to_writer(datafile, self).context("can not serialize repository data to datafile")?;

        Ok(())
    }

    fn move_file_to_object_store<P: AsRef<Path> + Debug>(&self, path: P, file: &RepoFile) -> Result<(), Error> {
        let mut objects_path = self.get_objects_path();
        if file.is_dir || file.is_symlink {
            return Ok(());
        }

        let hash = file.hash.clone().ok_or_else(|| {
            format_err!(
                "file {:?} does not have a a hash. this should never happen",
                path
            )
        })?;

        let mut chunker = Chunker::new(hash.as_str(), 2);

        for _sublayer in { 0..self.sublayers } {
            objects_path = objects_path.join(chunker
                .next()
                .ok_or_else(|| format_err!("chunker for path {:?} has no avaible chunks", path))?);

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
