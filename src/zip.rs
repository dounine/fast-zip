use crate::directory::{CompressionType, Directory, Extra, ZipFile};
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
    fn add_directory(&mut self, file_name: &str) -> Result<(), ZipError> {
        // self.directories.retain(|v| v.file_name != file_name);
        let file_name_length = file_name.as_bytes().len() as u16;
        let mut directory = Directory {
            compressed: true,
            data: None,
            version: 798,
            min_version: 10,
            bit_flag: 0,
            compression_method: CompressionType::Store,
            last_modification_time: 0,
            last_modification_date: 0,
            crc_32_uncompressed_data: 0,
            compressed_size: 0,
            uncompressed_size: 0,
            file_name_length,
            extra_field_length: 0,
            file_comment_length: 0,
            number_of_starts: 0,
            internal_file_attributes: 0,
            external_file_attributes: 1106051088,
            offset_of_local_file_header: 0,
            file_name: file_name.to_string(),
            extra_fields: vec![
                Extra::UnixExtendedTimestamp {
                    mtime: Some(0),
                    atime: None,
                    ctime: None,
                },
                Extra::UnixAttrs { uid: 503, gid: 20 },
            ],
            file_comment: vec![],
            file: Some(ZipFile {
                min_version: 10,
                bit_flag: 0,
                compression_method: CompressionType::Deflate,
                last_modification_time: 0,
                last_modification_date: 0,
                crc_32_uncompressed_data: 0,
                compressed_size: 0,
                uncompressed_size: 0,
                file_name_length,
                extra_field_length: 0,
                file_name: file_name.to_string(),
                extra_fields: vec![
                    Extra::UnixExtendedTimestamp {
                        mtime: Some(0),
                        atime: Some(0),
                        ctime: None,
                    },
                    Extra::UnixAttrs { uid: 503, gid: 20 },
                ],
                data_position: 0,
            }),
        };
        let mut extra_field_length = 0;
        for extra_field in &directory.extra_fields {
            extra_field_length += extra_field.size(true);
        }
        directory.extra_field_length = extra_field_length;
        let mut extra_field_length = 0;
        if let Some(file) = &mut directory.file {
            for extra_field in &file.extra_fields {
                extra_field_length += extra_field.size(false);
            }
            file.extra_field_length = extra_field_length;
        }
        self.directories.push(directory);
        Ok(())
    }

    pub fn add_file_and_compress(&mut self, data: Stream, file_name: &str) -> Result<(), ZipError> {
        let folders: Vec<&str> = file_name.split("/").collect();
        if folders.len() > 1 {
            // 一层一层创建文件夹
            let folders: Vec<String> = folders
                .iter()
                .take(folders.len() - 1)
                .scan(vec![], |acc, &ext| {
                    acc.push(ext);
                    Some(acc.join("/"))
                })
                .collect();
            let dirs: Vec<String> = self
                .directories
                .iter()
                .filter(|v| v.file_name.ends_with("/"))
                .map(|v| v.file_name.clone())
                .collect();
            for folder in folders {
                let path = format!("{}/", folder);
                if dirs.iter().find(|&v| *v == path).is_none() {
                    self.add_directory(&path)?;
                }
            }
        }
        self.directories.retain(|v| v.file_name != file_name);
        let file_name_length = file_name.as_bytes().len() as u16;
        let uncompressed_size = data.length() as u32;
        let crc_32_uncompressed_data = data.crc32_value()?;
        let compressed_size = data.compress(CompressionLevel::DefaultLevel)? as u32;
        let mut directory = Directory {
            compressed: true,
            data: Some(data),
            version: 798,
            min_version: 20,
            bit_flag: 0,
            compression_method: CompressionType::Deflate,
            last_modification_time: 0,
            last_modification_date: 0,
            crc_32_uncompressed_data,
            compressed_size,
            uncompressed_size,
            file_name_length,
            extra_field_length: 0,
            file_comment_length: 0,
            number_of_starts: 0,
            internal_file_attributes: 1,
            external_file_attributes: 2175008768,
            offset_of_local_file_header: 0,
            file_name: file_name.to_string(),
            extra_fields: vec![
                Extra::UnixExtendedTimestamp {
                    mtime: Some(1736154637),
                    atime: None,
                    ctime: None,
                },
                Extra::UnixAttrs { uid: 503, gid: 20 },
            ],
            file_comment: vec![],
            file: Some(ZipFile {
                min_version: 20,
                bit_flag: 0,
                compression_method: CompressionType::Deflate,
                last_modification_time: 0,
                last_modification_date: 0,
                crc_32_uncompressed_data,
                compressed_size,
                uncompressed_size,
                file_name_length,
                extra_field_length: 0,
                file_name: file_name.to_string(),
                extra_fields: vec![
                    Extra::UnixExtendedTimestamp {
                        mtime: Some(1736154637),
                        atime: Some(1736195293),
                        ctime: None,
                    },
                    Extra::UnixAttrs { uid: 503, gid: 20 },
                ],
                data_position: 0,
            }),
        };
        let mut extra_field_length = 0;
        for extra_field in &directory.extra_fields {
            extra_field_length += extra_field.size(true);
        }
        directory.extra_field_length = extra_field_length;
        let mut extra_field_length = 0;
        if let Some(file) = &mut directory.file {
            for extra_field in &file.extra_fields {
                extra_field_length += extra_field.size(false);
            }
            file.extra_field_length = extra_field_length;
        }
        self.directories.push(directory);
        Ok(())
    }
    fn computer(&mut self) -> Result<(), ZipError> {
        let mut files_size = 0;
        let mut directors_size = 0;
        self.directories
            .sort_by(|a, b| a.file_name.cmp(&b.file_name));
        for director in &mut self.directories {
            director.offset_of_local_file_header = files_size as u32;
            director.exec()?;
            if let Some(file) = &mut director.file {
                files_size += file.size(false) + director.compressed_size as usize;
            }
            directors_size += director.size(true);
        }
        if let Some(eo_cd) = &mut self.eo_cd {
            eo_cd.size = directors_size as u32;
            eo_cd.entries = self.directories.len() as u16;
            eo_cd.offset = files_size as u32;
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
        let mut count = 0;
        for director in &mut self.directories {
            count += 1;
            if let Some(file) = director.file.take() {
                println!("{}", director.file_name);
                let position = file.data_position;
                let mut data = director.take_data(position, &mut self.stream)?;
                let stream = file.write(&endian)?;
                output.merge(stream)?;
                // if count <= 1 {
                    output.write(&mut data)?;
                // }
            }
            // if count >= 1 {
            //     break;
            // }
        }
        let directories = std::mem::take(&mut self.directories);
        for director in directories {
            let data = director.write_args(&endian, &Some(true))?;
            output.merge(data)?;
        }
        if let Some(eo_cd) = self.eo_cd.take() {
            let data = eo_cd.write(&endian)?;
            output.merge(data)?;
        }
        Ok(())
    }
}
