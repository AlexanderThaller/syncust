#[derive(Debug, Default)]
pub struct Chunker {
    chunks: Vec<String>,
}

impl Chunker {
    pub fn new(string: &str, chunk_size: usize) -> Chunker {
        let mut counter = 0;
        let mut collector = String::new();
        let mut chunks = Vec::default();

        for chr in string.chars() {
            collector = format!("{}{}", collector, chr);
            counter += 1;

            if counter == chunk_size {
                chunks.push(collector.clone());
                collector = String::new();
                counter = 0;
            }
        }

        chunks.reverse();

        Chunker { chunks: chunks }
    }
}

impl Iterator for Chunker {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.pop()
    }
}
