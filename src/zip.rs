use crate::directory::Directory;
use crate::eocd::EoCd;
use crate::error::ZipError;
use fast_stream::bytes::ValueWrite;
use fast_stream::endian::Endian;
use fast_stream::stream::Stream;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub struct Zip<T> {
    stream: Stream<T>,
    pub eo_cd: Option<EoCd>,
    pub directories: Vec<Directory>,
}
impl<T: Read + Write + Seek> Zip<T> {
    pub fn new(stream: Stream<T>) -> Self {
        Self {
            stream,
            eo_cd: None,
            directories: vec![],
        }
    }
    pub fn parse(&mut self) -> Result<(), ZipError> {
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
    fn computer(&mut self) -> Result<(), ZipError> {
        let mut files_size = 0;
        let mut directors_size = 0;
        for director in &self.directories {
            files_size += director.file.size();
            directors_size += director.size();
            dbg!(director.file.size());
            dbg!(director.size());
        }
        if let Some(eocd) = &mut self.eo_cd{
            eocd.size = directors_size as u32;
            eocd.entries = self.directories.len() as u16;
            eocd.offset = files_size as u32;
        }

        Ok(())
    }
    pub fn write<O: Read + Write + Seek>(&mut self, output: &mut O) -> Result<(), ZipError> {
        let endian = Endian::Little;
        self.computer()?;
        for director in &mut self.directories {
            output.write(&director.write(&endian)?)?;
        }
        if let Some(eo_cd) = &self.eo_cd {
            output.write(&eo_cd.write(&endian)?)?;
        }
        Ok(())
    }
}
