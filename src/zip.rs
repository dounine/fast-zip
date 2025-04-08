use crate::directory::{CompressionType, Directory, ZipFile};
use crate::eocd::EoCd;
use crate::error::ZipError;
use fast_stream::bytes::{Bytes, ValueWrite};
use fast_stream::crc32::CRC32;
use fast_stream::deflate::{CompressionLevel, Deflate};
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
    pub fn add_directory(&mut self, data: Stream, file_name: &str) -> Result<(), ZipError> {
        let file_name_length = file_name.as_bytes().len() as u16;
        let data_len = data.length();
        let crc_32_uncompressed_data = data.crc32_value()? & 0xFFFFFFFF;
        let compressed_size = data.compress(CompressionLevel::DefaultLevel)? as u32;
        self.directories.push(Directory {
            data: Some(data),
            version: 0,
            min_version: 20,
            bit_flag: 0,
            compression_method: CompressionType::Deflate,
            last_modification_time: 0,
            last_modification_date: 0,
            crc_32_uncompressed_data,
            compressed_size,
            uncompressed_size: data_len as u32,
            file_name_length,
            extra_field_length: 0,
            file_comment_length: 0,
            number_of_starts: 0,
            internal_file_attributes: 0,
            external_file_attributes: 0,
            offset_of_local_file_header: 0,
            file_name: file_name.to_string(),
            extra_field: vec![],
            file_comment: vec![],
            file: ZipFile {
                min_version: 20,
                bit_flag: 0,
                compression_method: CompressionType::Deflate,
                last_modification_time: 0,
                last_modification_date: 0,
                crc_32_uncompressed_data,
                compressed_size,
                uncompressed_size: data_len as u32,
                file_name_length,
                extra_field_length: 0,
                file_name: file_name.to_string(),
                extra_field: vec![],
                data_position: 0,
            },
        });
        Ok(())
    }
    fn computer(&mut self) -> Result<(), ZipError> {
        let mut files_size = 0;
        let mut directors_size = 0;
        for director in &mut self.directories {
            director.offset_of_local_file_header = files_size as u32;
            files_size += director.file.size() + director.compressed_size as usize;
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
        self.directories.iter().find(|e| e.file_name == file_name)
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
