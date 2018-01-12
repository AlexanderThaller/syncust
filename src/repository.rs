use crossbeam_channel::unbounded;
use failure::{
    Error,
    ResultExt,
};
use index::Index;
use num_cpus;
use pathclassifier;
use pathclassifier::PathType;
use repofile::RepoFile;
use repostatus::RepoStatus;
use serde_json::{
    from_reader,
    to_writer,
};
use std::fmt::Debug;
use std::fs::{
    create_dir_all,
    File,
};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{
    Arc,
    Barrier,
    Mutex,
};
use std::thread;
use time::PreciseTime;
use walkdir::WalkDir;

#[derive(Debug, Fail)]
enum RepositoryError {
    #[fail(display = "repository is already initialized")] AlreadyInitialized,
    #[fail(display = "repository is not initialized")] NotInitialized,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    sublayers: usize,
    version: usize,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            sublayers: 4,
            version: 1,
        }
    }
}

#[derive(Debug)]
pub struct Repository {
    path: PathBuf,
    settings: Settings,
}

impl Default for Repository {
    fn default() -> Repository {
        Repository {
            path: PathBuf::new(),
            settings: Settings::default(),
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

    pub fn open<P: AsRef<Path> + Debug>(path: P) -> Result<Repository, Error> {
        let mut repository = Repository::default().with_path(path);

        if !repository.is_inialized() {
            Err(RepositoryError::NotInitialized)?
        }

        repository.load_settings().context("can not load settings")?;

        Ok(repository)
    }

    pub fn init(&self) -> Result<(), Error> {
        if self.is_inialized() {
            Err(RepositoryError::AlreadyInitialized)?
        }
        create_dir_all(self.get_data_path()).context("can not create data dir")?;

        self.write_settings().context("can not write repo data")?;

        let _ = Index::open(self.get_index_path())?;

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

    pub fn add<P: AsRef<Path> + Debug>(&mut self, paths_to_add: Vec<P>) -> Result<(), Error> {
        if !self.is_inialized() {
            Err(RepositoryError::NotInitialized)?
        }

        for path in paths_to_add {
            trace!("repository::Repository::add: path - {:?}", path);

            self.add_folder(path)?;
        }

        debug!("finished adding files");

        Ok(())
    }

    pub fn status(&self) -> Result<RepoStatus, Error> {
        let index = Index::open(self.get_index_path())?;

        let mut status = RepoStatus::default();
        status.paths_count = index.count();

        Ok(status)
    }

    fn add_folder<P: AsRef<Path> + Debug>(&self, folder_path: P) -> Result<(), Error> {
        let repo_path = self.path.clone();
        let data_path = self.get_data_path();
        let (tx, rx) = unbounded();

        let worker = num_cpus::get() - 1;
        let index = Index::open(self.get_index_path())?;
        let mindex = Arc::new(Mutex::new(index));
        let barrier = Arc::new(Barrier::new(worker + 1));

        for worker in 0..worker {
            let rx = rx.clone();
            let repo_path = repo_path.clone();
            let mindex = Arc::clone(&mindex);
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                let repo = Repository::open(&repo_path).expect("can not open worker repository");

                loop {
                    let entry = rx.recv();
                    debug!("worker {} received message", worker);

                    if entry.is_err() {
                        debug!("worker {} has ended", worker);
                        break;
                    }

                    if let Err(err) = repo.add_file(&mindex, entry.unwrap()) {
                        error!("{:?}", err)
                    }
                }

                debug!("worker thread {} is waiting", worker);
                barrier.wait();
            });
        }

        for entry in WalkDir::new(folder_path) {
            let path = entry.unwrap().path().to_path_buf();

            if path == repo_path {
                continue;
            }

            if path.starts_with(&data_path) {
                continue;
            }

            tx.send(path).expect("can not send path");
        }

        debug!("dropping tx channel");
        drop(tx);

        debug!("main thread is waiting");
        barrier.wait();

        Ok(())
    }

    fn add_file<P: AsRef<Path> + Debug>(&self, index: &Arc<Mutex<Index>>, file_path: P) -> Result<(), Error> {
        if file_path.as_ref().starts_with(self.get_data_path()) {
            bail!("can not add file that is inside the data dir")
        }

        debug!("add_file: adding file {:?}", file_path);

        let start = PreciseTime::now();

        let path = if file_path.as_ref().starts_with(&self.path) {
            file_path
                .as_ref()
                .strip_prefix(&self.path)
                .expect("can not strip repo path")
                .to_path_buf()
        } else {
            file_path.as_ref().to_path_buf()
        };

        trace!("add_file: path - {:?}", path);

        debug!("add_file: checking if path is already tracked");
        if index.lock().unwrap().contains(&path) {
            warn!("file {:?} is already tracked by the repo", file_path);
            return Ok(());
        }

        debug!("add_file: creating repo_file from file_path");
        let file = RepoFile::from_path(&file_path).context(format_err!(
            "can not
        create file from path {:?}",
            file_path
        ))?;

        let checking = PreciseTime::now();

        index.lock().unwrap().set(path, &file)?;
        let index = PreciseTime::now();

        debug!(
            "added file {:?}: checking - {:?}, index - {:?}",
            file_path,
            start.to(checking),
            checking.to(index),
        );

        Ok(())
    }

    fn clone_local<P: AsRef<Path> + Debug>(&mut self, _source_path: P) -> Result<(), Error> {
        unimplemented!()
    }

    fn write_settings(&self) -> Result<(), Error> {
        let settings_path = self.get_settings_path();
        let settings_file = File::create(&settings_path).context(format_err!(
            "can not create settings file {:?}",
            settings_path
        ))?;

        to_writer(&settings_file, &self.settings).context("can not serialize repository settings to settings file")?;

        Ok(())
    }

    fn load_settings(&mut self) -> Result<(), Error> {
        let settings_file = File::open(self.get_settings_path()).context("can not open settings file")?;
        let settings: Settings = from_reader(settings_file).context("can not deserialize settings")?;

        self.settings = settings;

        Ok(())
    }

    fn get_data_path(&self) -> PathBuf {
        self.path.clone().join(".syncust")
    }

    fn get_index_path(&self) -> PathBuf {
        self.get_data_path().join("index.rocksdb")
    }

    fn get_settings_path(&self) -> PathBuf {
        self.get_data_path().join("settings.json")
    }

    fn is_inialized(&self) -> bool {
        self.get_data_path().exists()
    }
}
