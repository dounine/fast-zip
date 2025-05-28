use crate::directory::{CompressionType, DataDescriptor, Directory, Extra, ZipFile};
use crate::eocd::EoCd;
use crate::error::ZipError;
use fast_stream::bytes::{Bytes, ValueWrite};
use fast_stream::deflate::CompressionLevel;
use fast_stream::endian::Endian;
use fast_stream::pin::Pin;
use fast_stream::stream::Stream;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct Zip {
    pub stream: Option<Stream>,
    stream_size: usize,
    crc32_computer: bool,
    pub eo_cd: Option<EoCd>,
    pub write_clear: bool,
    pub compression_level: CompressionLevel,
    pub directories: IndexMap<String, Directory>,
}
impl Zip {
    pub fn with_crc32(&mut self, value: bool) {
        self.crc32_computer = value;
    }
    pub fn size(&self) -> usize {
        self.stream_size
    }
    pub fn new(stream: Stream) -> Result<Self, ZipError> {
        let mut info = Self {
            stream_size: stream.length() as usize,
            stream: Some(stream),
            eo_cd: None,
            write_clear: true,
            crc32_computer: true,
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
        if let Some(mut stream) = std::mem::take(&mut self.stream) {
            let eo_cd = stream.read_value::<EoCd>()?;
            stream.set_position(eo_cd.offset as u64)?;
            let mut directories = IndexMap::with_capacity(eo_cd.entries as usize);
            for _ in 0..eo_cd.entries {
                let dir: Directory = stream.read_value()?;
                directories.insert(dir.file_name.clone(), dir);
            }
            self.directories = directories;
            self.eo_cd = Some(eo_cd);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn add_directory(&mut self, file_name: &str) -> Result<(), ZipError> {
        let file_name_length = file_name.as_bytes().len() as u16;
        let mut directory = Directory {
            compressed: true,
            data: Stream::empty(),
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
            file: ZipFile {
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
            },
        };
        let mut extra_field_length = 0;
        for extra_field in &directory.extra_fields {
            extra_field_length += extra_field.size(true);
        }
        directory.extra_field_length = extra_field_length;
        let mut extra_field_length = 0;
        for extra_field in &directory.file.extra_fields {
            extra_field_length += extra_field.size(false);
        }
        directory.file.extra_field_length = extra_field_length;

        self.directories
            .insert(directory.file_name.clone(), directory);
        Ok(())
    }
    pub fn remove_file(&mut self, file_name: &str) {
        self.directories.swap_remove(file_name);
    }
    pub fn save_file(&mut self, data: Stream, file_name: &str) -> Result<(), ZipError> {
        if let Some(dir) = self.directories.get_mut(file_name) {
            dir.put_data(data);
            return Ok(());
        }
        self.add_file(data, file_name)
    }
    pub fn add_file(&mut self, data: Stream, file_name: &str) -> Result<(), ZipError> {
        let file_name_length = file_name.as_bytes().len() as u16;
        let uncompressed_size = data.length() as u32;
        let crc_32_uncompressed_data = 0; //data.crc32_value();
        let compressed_size = uncompressed_size; //data.compress(CompressionLevel::DefaultLevel)? as u32;
        let mut directory = Directory {
            compressed: false,
            data,
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
            file: ZipFile {
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
            },
        };
        let mut extra_field_length = 0;
        for extra_field in &directory.extra_fields {
            extra_field_length += extra_field.size(true);
        }
        directory.extra_field_length = extra_field_length;
        let mut extra_field_length = 0;
        for extra_field in &directory.file.extra_fields {
            extra_field_length += extra_field.size(false);
        }
        directory.file.extra_field_length = extra_field_length;
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
    fn computer(&mut self, callback: &mut impl FnMut(usize)) -> Result<bool, ZipError> {
        let mut files_size = 0;
        let mut directors_size = 0;
        // self.directories
        //     .sort_keys();
        for (_, director) in &mut self.directories {
            director.offset_of_local_file_header = files_size as u32;
            director.exec(self.crc32_computer, &self.compression_level, callback)?;
            files_size += director.file.size(false) + director.compressed_size as usize;
            directors_size += director.size(true);
        }
        if let Some(eo_cd) = &mut self.eo_cd {
            eo_cd.size = directors_size as u32;
            eo_cd.entries = self.directories.len() as u16;
            eo_cd.offset = files_size as u32;
        }
        Ok(false)
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
        let mut header_stream = output.copy_empty()?;
        if self.write_clear {
            for (_, mut director) in std::mem::take(&mut self.directories) {
                let mut file = director.file.clone();
                let mut data_descriptor = file.data_descriptor.take();
                let mut data = &mut director.data;
                let mut stream = file.write(&endian)?;
                stream.seek_start()?;
                output.append(&mut stream)?;
                data.seek_start()?;
                output.append(&mut data)?;
                if let Some(data_descriptor) = data_descriptor.take() {
                    output.write_value(data_descriptor)?;
                }
                let mut data = director.write_args(&endian, &Some(true))?;
                data.seek_start()?;
                header_stream.append(&mut data)?;
            }
        } else {
            for (_, director) in &mut self.directories {
                let mut file = director.file.clone();
                let mut data_descriptor = file.data_descriptor.take();
                let mut data = &mut director.data;
                let mut stream = file.write(&endian)?;
                director.file.data_descriptor = data_descriptor.clone();
                stream.seek_start()?;
                output.append(&mut stream)?;
                data.seek_start()?;
                output.append(&mut data)?;
                if let Some(data_descriptor) = data_descriptor.take() {
                    output.write_value(data_descriptor)?;
                }
                let mut data = director
                    .clone_not_stream()
                    .write_args(&endian, &Some(true))?;
                data.seek_start()?;
                header_stream.append(&mut data)?;
            }
        }
        header_stream.seek_start()?;
        output.append(&mut header_stream)?;
        let mut eo_cd = if self.write_clear {
            self.eo_cd.take()
        } else {
            if let Some(eo_cd) = &self.eo_cd {
                Some(eo_cd.clone())
            } else {
                None
            }
        };
        if let Some(eo_cd) = eo_cd.take() {
            let mut data = eo_cd.write(&endian)?;
            data.seek_start()?;
            output.append(&mut data)?;
        }
        Ok(())
    }
}
