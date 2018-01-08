use chunker::Chunker;
use failure::{
    Error,
    ResultExt,
};
use pathclassifier;
use pathclassifier::PathType;
use pathdiff::diff_paths;
use rayon::prelude::*;
use repofile::RepoFile;
use serde_json::{
    from_reader,
    to_writer,
};
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

type Files = BTreeMap<PathBuf, RepoFile>;

#[derive(Debug, Fail)]
enum RepositoryError {
    #[fail(display = "repository is already initialized")] AlreadyInitialized,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    #[serde(skip)] path: PathBuf,
    files: Files,
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
    pub fn with_path<P: AsRef<Path> + Debug>(self, path: P) -> Repository {
        Repository {
            path: path.as_ref().to_path_buf(),
            ..self
        }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        if self.get_data_path().exists() {
            Err(RepositoryError::AlreadyInitialized)?
        }

        // TODO: Make this more efficient so it wont blow up ram when there are a lot
        // of files maybe make a channel queue or something
        {
            let paths: Vec<_> = WalkDir::new(&self.path)
                .into_iter()
                .map(|entry| entry.unwrap())
                .map(|entry| entry.path().to_path_buf())
                .collect();

            info!("Collected {} paths", paths.len());

            let entries: Vec<_> = paths
                .par_iter()
                .filter(|path| path != &&self.path)
                .map(|path| {
                    (
                        RepoFile::from_path(&path),
                        path.strip_prefix(&self.path)
                            .expect("path of entry does not have repo as prefix. this should never happen")
                            .to_path_buf(),
                    )
                })
                .collect();

            info!("Finished processing paths");

            for entry in entries {
                trace!("repository::Repository::init: entry - {:?}", entry);
                let file = entry.0?;

                self.files.insert(entry.1.to_path_buf(), file);
            }
        }

        create_dir_all(self.get_data_path()).context("can not create data dir")?;

        info!("Moving files");

        for (path, file) in &self.files {
            self.move_file_to_object_store(path, file)?;
        }

        info!("Writing repo data");

        self.write_repodata().context("can not write repo data")?;

        Ok(())
    }

    pub fn clone<P: AsRef<Path> + Debug>(&mut self, source_path: P) -> Result<(), Error> {
        if self.path.is_dir() {
            bail!("destination dir does already exist, refusing to continue")
        }
        create_dir_all(&self.path).context("can not create destnation dir")?;

        self.init().context("can not initialize destination dir")?;

        match pathclassifier::from_path(&source_path).context("can not classify source path")? {
            PathType::Local => self.clone_local(source_path)?,
        }

        Ok(())
    }

    fn clone_local<P: AsRef<Path> + Debug>(&mut self, source_path: P) -> Result<(), Error> {
        // Steps:
        // . Copy source_paths data.json
        // . Create remote based on data.json
        // . Create symlinks based on that data.json

        let source_repo = Repository::default().with_path(source_path);

        let destination_data_file = self.get_data_file();
        let source_data_file = source_repo.get_data_file();

        trace!(
            "repository::Repository::clone_local: destination_data_file - {:?}, source_data_file - {:?}",
            destination_data_file,
            source_data_file
        );

        let source_data = File::open(source_data_file).context("can not open destination file")?;
        let repodata: Repository = from_reader(source_data).context("can not deserialize destination data from file")?;

        trace!(
            "repository::Repository::clone_local: repodata.files - {:#?}",
            repodata.files
        );

        self.populate_directories(&repodata.files)
            .context("can not populate files")?;

        self.populate_files(&repodata.files)
            .context("can not populate files")?;

        self.files = repodata.files;

        self.write_repodata().context("can not write repodata")?;

        Ok(())
    }

    fn populate_directories(&self, files: &Files) -> Result<(), Error> {
        let errors: Vec<_> = files
            .iter()
            .filter(|&(_, entry)| entry.is_dir)
            .map(|(directory, _)| {
                trace!(
                    "repository::Repository::populate_directories: directory - {:?}",
                    directory
                );

                create_dir_all(self.get_full_path(&directory)).context(format_err!("can not create directory {:?}", directory))
            })
            .filter(|err| err.is_err())
            .map(|err| err.err().unwrap())
            .collect();

        trace!(
            "repository::Repository::populate_directories: errors - {:#?}",
            errors
        );

        for error in &errors {
            error!("{:?}", error)
        }

        if !errors.is_empty() {
            bail!("can not populate directories")
        }

        Ok(())
    }

    fn populate_files(&self, files: &Files) -> Result<(), Error> {
        let errors: Vec<_> = files
            .iter()
            .filter(|&(_, entry)| !entry.is_dir)
            .map(|(path, file)| self.populate_file(self.get_full_path(&path), file))
            .filter(|err| err.is_err())
            .map(|err| err.err().unwrap())
            .collect();

        trace!(
            "repository::Repository::populate_files: errors - {:#?}",
            errors
        );

        for error in &errors {
            let cause: Vec<_> = error.causes().map(|cause| format!("{}", cause)).collect();
            error!("{}", cause.join(": "))
        }

        if !errors.is_empty() {
            bail!("can not populate files")
        }

        Ok(())
    }

    fn populate_file<P: AsRef<Path> + Debug>(&self, path: P, file: &RepoFile) -> Result<(), Error> {
        let objects_path = self.objects_path_from_file(file)?;

        debug!(
            "repository::Repository::populate_file: path - {:?}, objects_path - {:?}",
            path, objects_path
        );

        let relative_path =
            diff_paths(&objects_path, path.as_ref()).ok_or_else(|| format_err!("can not get relative path for file path {:?}", path))?;

        debug!(
            "repository::Repository::populate_file: relative_path - {:?}",
            relative_path
        );

        let relative_path = relative_path
            .strip_prefix("..")
            .context("can not strip unneded \"..\" prefix. this should never happen")?;

        symlink(&relative_path, &path).context(format_err!(
            "can not create symlink {:?} -> {:?}",
            path,
            relative_path,
        ))?;

        Ok(())
    }

    fn write_repodata(&self) -> Result<(), Error> {
        let data_path = self.get_data_file();
        let datafile = File::create(&data_path).context(format_err!("can not create datafile {:?}", data_path))?;

        to_writer(datafile, self).context("can not serialize repository data to datafile")?;

        Ok(())
    }

    fn move_file_to_object_store<P: AsRef<Path> + Debug>(&self, path: P, file: &RepoFile) -> Result<(), Error> {
        if file.is_dir || file.is_symlink {
            return Ok(());
        }

        let objects_path = self.objects_path_from_file(file)?;

        debug!(
            "repository::Repository::move_file_to_object_store: path - {:?}, file - {:?}, objects_path - {:?}",
            path, file, objects_path
        );

        let fullpath = self.get_full_path(&path);
        let parent = objects_path
            .parent()
            .ok_or_else(|| format_err!("can not get parent for path {:?}", path))?;

        create_dir_all(parent)?;
        rename(&fullpath, &objects_path).context(format_err!(
            "can not move file from {:?} to {:?}",
            path,
            objects_path
        ))?;

        self.populate_file(fullpath, file)?;

        Ok(())
    }

    fn objects_path_from_file(&self, file: &RepoFile) -> Result<PathBuf, Error> {
        if file.is_dir || file.is_symlink {
            bail!("can not create object path for directories or symlinks")
        }

        let mut objects_path = self.get_objects_path();

        let hash = file.hash.clone().ok_or_else(|| {
            format_err!(
                "file {:?} does not have a a hash. this should never happen",
                file
            )
        })?;

        let mut chunker = Chunker::new(hash.as_str(), 2);

        for _sublayer in { 0..self.sublayers } {
            objects_path = objects_path.join(chunker
                .next()
                .ok_or_else(|| format_err!("chunker for file {:?} has no avaible chunks", file))?);
        }

        Ok(objects_path.join(hash))
    }

    fn get_data_path(&self) -> PathBuf {
        self.path.clone().join(".syncust")
    }

    fn get_objects_path(&self) -> PathBuf {
        self.get_data_path().join("objects")
    }

    fn get_data_file(&self) -> PathBuf {
        self.get_data_path().join("data.json")
    }

    fn get_full_path<P: AsRef<Path> + Debug>(&self, path: P) -> PathBuf {
        self.path.join(path)
    }
}
