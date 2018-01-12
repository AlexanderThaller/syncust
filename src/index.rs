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
use std::fmt::Debug;
use std::path::Path;

pub struct Index {
    db: DB,
}

impl Index {
    pub fn open<P: AsRef<Path> + Debug>(path: P) -> Result<Index, Error> {
        let db = DB::open_default(path)?;
        Ok(Index { db: db })
    }

    pub fn set<P: AsRef<Path> + Debug>(&self, path: P, file: &RepoFile) -> Result<(), Error> {
        let key: Vec<u8> = serialize(&path.as_ref(), Infinite).context("can not serialize key to bytes")?;
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
}
