use crate::directory::Directory;
use crate::eocd::EoCd;
use crate::error::ZipError;
use fast_stream::bytes::{Bytes, ValueWrite};
use fast_stream::endian::Endian;
use fast_stream::stream::Stream;
use std::io::{Seek, SeekFrom, Write};

#[derive(Debug)]
pub struct Zip {
    pub stream: Stream,
    pub eo_cd: Option<EoCd>,
    pub directories: Vec<Directory>,
}
impl Zip {
    pub fn new(stream: Stream) -> Self {
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
        for director in &mut self.directories {
            director.offset_of_local_file_header = files_size as u32;
            files_size += director.file.size();
            directors_size += director.size();
        }
        if let Some(eocd) = &mut self.eo_cd {
            eocd.size = directors_size as u32;
            eocd.entries = self.directories.len() as u16;
            eocd.offset = files_size as u32;
        }

        Ok(())
    }
    pub fn get_mut(&mut self, file_name: &str) -> Option<&mut Directory> {
        self.directories
            .iter_mut()
            .find(|e| e.file_name == file_name)
    }
    pub fn get(&mut self, file_name: &str) -> Option<&Directory> {
        self.directories
            .iter()
            .find(|e| e.file_name == file_name)
    }
    pub fn write(&mut self, output: &mut Stream) -> Result<(), ZipError> {
        let endian = Endian::Little;
        self.computer()?;
        for director in &mut self.directories {
            let extra_field = std::mem::take(&mut director.file.extra_field);
            let data = director.file.write(&endian)?;
            output.merge(data)?;
            output.write(&extra_field)?;
            let mut data = director.origin_data(&mut self.stream)?;
            output.write(&mut data)?;
        }
        for director in &mut self.directories {
            let data = director.write(&endian)?;
            output.merge(data)?;
        }
        if let Some(eo_cd) = &self.eo_cd {
            let data = eo_cd.write(&endian)?;
            output.merge(data)?;
        }
        Ok(())
    }
}
