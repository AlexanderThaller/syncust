use bincode::{
    deserialize,
    serialize,
    Infinite,
};
use failure::{
    Error,
    ResultExt,
};
use repofile::RepoFile;
use rocksdb::{
    IteratorMode,
    DB,
};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

pub struct Index {
    db: DB,
}

impl Index {
    pub fn open<P: AsRef<Path> + Debug>(path: P) -> Result<Index, Error> {
        let db = DB::open_default(path)?;
        Ok(Index { db: db })
    }

    pub fn set<P: AsRef<Path> + Debug>(&self, path: P, file: &RepoFile) -> Result<(), Error> {
        let key: Vec<u8> = serialize(&path.as_ref(), Infinite).context(format_err!("can not serialize path {:?} to bytes", path))?;
        let data: Vec<u8> = serialize(&file, Infinite).context("can not serialize data to bytes")?;

        self.db.put(&key, &data)?;

        Ok(())
    }

    pub fn get<P: AsRef<Path> + Debug>(&self, path: P) -> Result<RepoFile, Error> {
        let key: Vec<u8> = serialize(&path.as_ref(), Infinite).context("can not serialize key to bytes")?;

        match self.db.get(&key)? {
            Some(data) => {
                let decoded: RepoFile = deserialize(&data)?;
                Ok(decoded)
            }
            None => bail!("key not found in index"),
        }
    }

    pub fn contains<P: AsRef<Path> + Debug>(&self, path: P) -> bool {
        debug!("contains: checking if index contains key {:?}", path);
        match serialize(&path.as_ref(), Infinite) {
            Ok(key) => {
                debug!("contains: serialized key to bytes trying to get from index");

                match self.db.get(&key) {
                    Ok(option) => match option {
                        Some(_) => true,
                        None => false,
                    },
                    Err(_) => false,
                }
            }
            Err(err) => {
                warn!("can not serialize key to bytes: {}", err);
                false
            }
        }
    }

    pub fn count(&self) -> usize {
        let iter = self.db.iterator(IteratorMode::Start);
        iter.count()
    }

    pub fn debug_tracked_files(&self) -> Result<(), Error> {
        let iter = self.db.iterator(IteratorMode::Start);

        for (key, data) in iter {
            let decoded_key: PathBuf = deserialize(&key)?;
            let decoded_data: RepoFile = deserialize(&data)?;

            println!("key: {:?}\nvalue: {:#?}", decoded_key, decoded_data);
        }

        Ok(())
    }

    pub fn entries(&self) -> Result<BTreeMap<PathBuf, RepoFile>, Error> {
        let iter = self.db.iterator(IteratorMode::Start);

        let mut out = BTreeMap::default();
        for (path, metadata) in iter {
            let decoded_path: PathBuf = deserialize(&path)?;
            let decoded_metadata: RepoFile = deserialize(&metadata)?;

            out.insert(decoded_path, decoded_metadata);
        }

        Ok(out)
    }
}
