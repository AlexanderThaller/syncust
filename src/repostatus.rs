use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct RepoStatus {
    pub paths_count: usize,
    pub untracked_paths: BTreeSet<PathBuf>,
}

impl fmt::Display for RepoStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Paths Tracked: {}\n", self.paths_count)?;

        if !self.untracked_paths.is_empty() {
            let untracked_paths = self.untracked_paths
                .iter()
                .fold(String::new(), |acc, x| format!("{}\t{:?}\n", acc, x));

            write!(f, "Untracked Paths:\n{}", untracked_paths)?;
        }

        Ok(())
    }
}
