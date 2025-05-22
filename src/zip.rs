use crate::directory::{CompressionType, DataDescriptor, Directory, Extra, ZipFile};
use crate::eocd::EoCd;
use crate::error::ZipError;
use fast_stream::bytes::{Bytes, ValueWrite};
use fast_stream::crc32::CRC32;
use fast_stream::deflate::CompressionLevel;
use fast_stream::endian::Endian;
use fast_stream::stream::Stream;
use indexmap::IndexMap;
use std::io::{Seek, SeekFrom, Write};

#[derive(Debug)]
pub struct Zip {
    pub stream: Stream,
    pub eo_cd: Option<EoCd>,
    pub compression_level: CompressionLevel,
    pub directories: IndexMap<String, Directory>,
}
impl Zip {
    pub fn new(stream: Stream) -> Result<Self, ZipError> {
        // let mut map = IndexMap::new();
        // map.insert("a", 1); // 保持插入顺序
        // map.insert("b", 2);
        let mut info = Self {
            stream,
            eo_cd: None,
            compression_level: CompressionLevel::DefaultLevel,
            directories: IndexMap::new(),
        };
        info.parse()?;
        Ok(info)
    }
    pub fn with_compression_level(&mut self, compression_level: CompressionLevel) {
        self.compression_level = compression_level
    }
    pub fn parse(&mut self) -> Result<(), ZipError> {
        let eo_cd = self.stream.read_value::<EoCd>()?;
        self.stream.seek(SeekFrom::Start(eo_cd.offset as u64))?;
        let mut directories = IndexMap::new();
        for _ in 0..eo_cd.entries {
            let dir: Directory = self.stream.read_value()?;
            directories.insert(dir.file_name.clone(), dir);
        }
        self.directories = directories;
        self.eo_cd = Some(eo_cd);

        Ok(())
    }

    #[allow(dead_code)]
    fn add_directory(&mut self, file_name: &str) -> Result<(), ZipError> {
        // self.directories.retain(|v| file_name != file_name);
        let file_name_length = file_name.as_bytes().len() as u16;
        let mut directory = Directory {
            compressed: true,
            data: None,
            version: 798,
            min_version: 10,
            flags: 0,
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
                flags: 0,
                compression_method: CompressionType::Store,
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
                data_descriptor: None,
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
        self.directories
            .insert(directory.file_name.clone(), directory);
        Ok(())
    }
    pub fn save_file(&mut self, data: Stream, file_name: &str) -> Result<(), ZipError> {
        if let Some(dir) = self.directories.get_mut(file_name) {
            dir.put_data(data);
            return Ok(());
        }
        self.add_file(data, file_name)
    }
    pub fn remove_file(&mut self, file_name: &str) {
        self.directories.swap_remove(file_name);
        // self.directories.retain(|v| v.file_name != file_name);
    }
    pub fn add_file(&mut self, data: Stream, file_name: &str) -> Result<(), ZipError> {
        // let folders: Vec<&str> = file_name.split("/").collect();
        // if folders.len() > 1 {
        //     // 一层一层创建文件夹
        //     let folders: Vec<String> = folders
        //         .iter()
        //         .take(folders.len() - 1)
        //         .scan(vec![], |acc, &ext| {
        //             acc.push(ext);
        //             Some(acc.join("/"))
        //         })
        //         .collect();
        //     let dirs: Vec<String> = self
        //         .directories
        //         .iter()
        //         .filter(|v| v.file_name.ends_with("/"))
        //         .map(|v| v.file_name.clone())
        //         .collect();
        //     for folder in folders {
        //         let path = format!("{}/", folder);
        //         if dirs.iter().find(|&v| *v == path).is_none() {
        //             self.add_directory(&path)?;
        //         }
        //     }
        // }
        // self.directories.retain(|v| v.file_name != file_name);
        let file_name_length = file_name.as_bytes().len() as u16;
        let uncompressed_size = data.length() as u32;
        let crc_32_uncompressed_data = data.crc32_value()?;
        let compressed_size = uncompressed_size; //data.compress(CompressionLevel::DefaultLevel)? as u32;
        let mut directory = Directory {
            compressed: false,
            data: Some(data),
            version: 798,
            min_version: 20,
            flags: 0x08,
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
                flags: 0x08,
                compression_method: CompressionType::Deflate,
                last_modification_time: 0,
                last_modification_date: 0,
                crc_32_uncompressed_data: 0,
                compressed_size: 0,
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
                data_descriptor: Some(DataDescriptor {
                    crc32: crc_32_uncompressed_data,
                    compressed_size,
                    uncompressed_size,
                }),
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
        self.directories
            .insert(directory.file_name.clone(), directory);
        Ok(())
    }
    fn computer_un_compress_size(&mut self) -> usize {
        let mut total_size = 0;
        for (_, director) in &mut self.directories {
            total_size += director.exec_un_compress_size();
        }
        total_size
    }
    fn computer(&mut self, callback: &mut impl FnMut(usize)) -> Result<(), ZipError> {
        let mut files_size = 0;
        let mut directors_size = 0;
        // self.directories
        //     .sort_by(|a, b| a.file_name.cmp(&b.file_name));
        for (_, director) in &mut self.directories {
            director.offset_of_local_file_header = files_size as u32;
            director.exec(&self.compression_level, callback)?;
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
        self.directories.get_mut(file_name)
        // self.directories
        //     .iter_mut()
        //     .find(|e| e.file_name == file_name)
    }
    pub fn get(&mut self, file_name: &str) -> Option<&Directory> {
        self.directories.get(file_name)
        // self.directories.iter().find(|e| e.file_name == file_name)
    }
    fn create_adapter<T: FnMut(usize, usize, String)>(
        total: usize,
        sum: &mut usize,
        mut f: T,
    ) -> impl FnMut(usize) + use<'_, T> {
        move |x| {
            *sum += x;
            f(
                total,
                *sum,
                format!("{:.2}%", (*sum as f64 / total as f64) * 100.0),
            )
        }
    }
    pub fn write(
        &mut self,
        output: &mut Stream,
        callback: &mut impl FnMut(usize, usize, String),
    ) -> Result<(), ZipError> {
        let endian = Endian::Little;
        let total_size = self.computer_un_compress_size();
        let mut binding = 0;
        let mut callback = Self::create_adapter(total_size, &mut binding, callback);
        self.computer(&mut callback)?;
        for (_, director) in &mut self.directories {
            if let Some(file) = director.file.take() {
                let mut file = file;
                let position = file.data_position;
                let mut data = director.take_data(position, &mut self.stream)?;
                let data_descriptor_data = file.data_descriptor.take();
                let mut stream = file.write(&endian)?;
                stream.seek_start()?;
                output.append(&mut stream)?;
                output.write(&mut data)?;
                if let Some(data_descriptor) = data_descriptor_data {
                    output.write_value(data_descriptor)?;
                }
            }
        }
        let directories = std::mem::take(&mut self.directories);
        for (_, director) in directories {
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
