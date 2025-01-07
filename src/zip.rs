use crate::directory::Directory;
use crate::eocd::EoCd;
use crate::error::ZipError;
use std::io::{Read, Seek, SeekFrom, Write};
use fast_stream::stream::Stream;

#[derive(Debug)]
pub struct Zip<T> {
    stream: Stream<T>,
    eo_cd: Option<EoCd>,
    directories: Vec<Directory>,
}
impl<T: Read + Write + Seek> Zip<T> {
    pub fn new(stream: Stream<T>) -> Self {
        Self {
            stream,
            eo_cd: None,
            directories: vec![],
        }
    }
    pub fn init(&mut self) -> Result<(), ZipError> {
        let eo_cd = self.stream.read_value::<EoCd>()?;
        self.stream.seek(SeekFrom::Start(eo_cd.offset as u64))?;
        let mut directories = vec![];
        for _ in 0..eo_cd.entries {
            let dir: Directory = self.stream.read_value()?;
            directories.push(dir);
        }
        self.directories = directories;
        self.eo_cd = Some(eo_cd);

        Ok(())
    }
}
