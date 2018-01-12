use std::fmt;

#[derive(Debug, Default)]
pub struct RepoStatus {
    pub paths_count: usize,
}

impl fmt::Display for RepoStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Paths Tracked: {}", self.paths_count)
    }
}
