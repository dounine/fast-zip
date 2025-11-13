use crate::extra::{Center, Extra};
use crate::magic::Magic;
use crate::zip::Parser;
use crate::zip_file::{DataDescriptor, ZipFile};
use fast_stream::bytes::{Bytes, ValueRead, ValueWrite};
use fast_stream::deflate::{CompressionLevel, Deflate};
use fast_stream::derive::NumToEnum;
use fast_stream::endian::Endian;
use fast_stream::enum_to_bytes;
use fast_stream::pin::Pin;
use fast_stream::stream::Stream;
use std::fmt::Debug;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};

pub trait Size {
    fn size(&self) -> usize;
}
#[repr(u16)]
#[derive(Debug, Clone, Default, PartialEq, NumToEnum)]
pub enum CompressionType {
    #[default]
    Store = 0x0000,
    Shrink = 0x0001,
    Implode = 0x0006,
    Deflate = 0x0008,
    Deflate64 = 0x0009,
    BZIP2 = 0x000C,
    LZMA = 0x000E,
    XZ = 0x005F,
    JPEG = 0x0060,
    WavPack = 0x0061,
    PPMd = 0x0062,
    AES = 0x0063,
}
impl CompressionType {
    pub const fn byte_size() -> usize {
        2
    }
}
enum_to_bytes!(CompressionType, u16);
impl Directory<Parser> {
    pub fn exec_un_compress_size(&mut self) -> usize {
        if !self.compressed && self.compression_method == CompressionType::Deflate {
            self.data.length() as usize
        } else {
            0
        }
    }
    pub fn exec(
        &mut self,
        crc32_computer: bool,
        compression_level: &CompressionLevel,
        callback: &mut impl FnMut(usize),
    ) -> Result<()> {
        if !self.compressed && self.compression_method == CompressionType::Deflate {
            let crc_32_uncompressed_data = if crc32_computer {
                self.data.init_crc32();
                self.data.hash_computer()?;
                self.data.crc32_value()
            } else {
                0
            };
            self.crc_32_uncompressed_data = crc_32_uncompressed_data; //crc32 设置为0也能安装，网页可以忽略计算加快速度
            self.file.crc_32_uncompressed_data = crc_32_uncompressed_data;
            self.data.seek_start()?;
            let compress_size = self.data.compress_callback(compression_level, callback)?;
            self.compressed_size = compress_size as u32;
            self.compressed = true;
            self.file.compressed_size = self.compressed_size;
            return Ok(());
        }
        self.data.seek_start()?;
        Ok(())
    }
    pub fn put_data(&mut self, stream: Stream) {
        self.compressed_size = stream.length() as u32;
        self.uncompressed_size = stream.length() as u32;
        // if let Some(file) = &mut self.file {
        self.file.compressed_size = self.compressed_size;
        self.file.uncompressed_size = self.uncompressed_size;
        // }
        self.compressed = false;
        self.data = stream
    }
    // pub fn put_data_and_compress(
    //     &mut self,
    //     stream: Stream,
    //     compression_level: &CompressionLevel,
    // ) -> Result<u64> {
    //     self.uncompressed_size = stream.length() as u32;
    //     let compress_size = stream.compress(compression_level)?;
    //     self.compressed_size = compress_size as u32;
    //     self.compressed = true;
    //     self.data = stream;
    //     Ok(compress_size)
    // }
    pub fn decompressed_callback(
        &mut self,
        callback_fun: &mut impl FnMut(usize),
    ) -> Result<&mut Stream> {
        self.data.seek_start()?;
        if self.compressed {
            self.data.decompress_callback(callback_fun)?;
            self.compressed = false;
        }
        Ok(&mut self.data)
    }
    pub fn decompressed(&mut self) -> Result<&mut Stream> {
        self.data.seek_start()?;
        if self.compressed {
            self.data.decompress()?;
            self.compressed = false;
        }
        Ok(&mut self.data)
    }
}

const DIRECTORY_HEADER_SIZE: usize = Magic::byte_size()
    + size_of::<u16>() * 6
    + size_of::<u32>() * 3
    + size_of::<u16>() * 5
    + size_of::<u32>() * 2;
#[derive(Debug, Clone)]
pub struct Directory<TYPE> {
    pub r#type: TYPE,
    pub data: Stream,
    pub compressed: bool,
    pub version: u16,
    pub min_version: u16,
    pub flags: u16,
    pub compression_method: CompressionType,
    pub last_modification_time: u16,
    pub last_modification_date: u16,
    pub crc_32_uncompressed_data: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_comment_length: u16,
    pub number_of_starts: u16,
    pub internal_file_attributes: u16,
    pub external_file_attributes: u32,
    pub offset_of_local_file_header: u32,
    pub file_name: String,
    pub extra_fields: Vec<Extra<Center>>,
    pub file_comment: Vec<u8>,
    pub file: ZipFile<TYPE>,
}
impl Directory<Parser> {
    pub fn clone_all(&mut self) -> Result<Self> {
        Ok(Directory {
            r#type: Parser,
            data: self.data.clone_stream()?,
            compressed: self.compressed,
            version: self.version,
            min_version: self.min_version,
            flags: self.flags,
            compression_method: self.compression_method.clone(),
            last_modification_time: self.last_modification_time,
            last_modification_date: self.last_modification_date,
            crc_32_uncompressed_data: self.crc_32_uncompressed_data,
            compressed_size: self.compressed_size,
            uncompressed_size: self.uncompressed_size,
            file_name_length: self.file_name_length,
            extra_field_length: self.extra_field_length,
            file_comment_length: self.file_comment_length,
            number_of_starts: self.number_of_starts,
            internal_file_attributes: self.internal_file_attributes,
            external_file_attributes: self.external_file_attributes,
            offset_of_local_file_header: self.offset_of_local_file_header,
            file_name: self.file_name.clone(),
            extra_fields: self.extra_fields.clone(),
            file_comment: self.file_comment.clone(),
            file: self.file.clone(),
        })
    }
    pub fn clone_not_stream(&self) -> Self {
        Directory {
            r#type: Parser,
            data: Stream::empty(),
            compressed: self.compressed,
            version: self.version,
            min_version: self.min_version,
            flags: self.flags,
            compression_method: self.compression_method.clone(),
            last_modification_time: self.last_modification_time,
            last_modification_date: self.last_modification_date,
            crc_32_uncompressed_data: self.crc_32_uncompressed_data,
            compressed_size: self.compressed_size,
            uncompressed_size: self.uncompressed_size,
            file_name_length: self.file_name_length,
            extra_field_length: self.extra_field_length,
            file_comment_length: self.file_comment_length,
            number_of_starts: self.number_of_starts,
            internal_file_attributes: self.internal_file_attributes,
            external_file_attributes: self.external_file_attributes,
            offset_of_local_file_header: self.offset_of_local_file_header,
            file_name: self.file_name.clone(),
            extra_fields: self.extra_fields.clone(),
            file_comment: self.file_comment.clone(),
            file: self.file.clone(),
        }
    }
}
impl Directory<Parser> {
    pub fn compressed(&self) -> bool {
        self.compressed
    }
    pub fn size(&self) -> usize {
        let mut bytes =
            DIRECTORY_HEADER_SIZE + self.file_name.as_bytes().len() + self.file_comment.len();
        for extra_field in &self.extra_fields {
            bytes += extra_field.size() as usize
        }
        bytes
    }
}
impl ValueWrite for Directory<Parser> {
    fn write_args<T: Sized>(self, endian: &Endian, _args: &Option<T>) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(Magic::Directory)?;
        stream.write_value(self.version)?;
        stream.write_value(self.min_version)?;
        stream.write_value(self.flags)?;
        stream.write_value(self.compression_method)?;
        stream.write_value(self.last_modification_time)?;
        stream.write_value(self.last_modification_date)?;
        stream.write_value(self.crc_32_uncompressed_data)?;
        stream.write_value(self.compressed_size)?;
        stream.write_value(self.uncompressed_size)?;
        stream.write_value(self.file_name_length)?;
        stream.write_value(self.extra_field_length)?;
        stream.write_value(self.file_comment_length)?;
        stream.write_value(self.number_of_starts)?;
        stream.write_value(self.internal_file_attributes)?;
        stream.write_value(self.external_file_attributes)?;
        stream.write_value(self.offset_of_local_file_header)?;
        stream.write(self.file_name.as_bytes())?;
        for extra_field in self.extra_fields {
            stream.write_value(extra_field)?;
        }
        stream.write(&self.file_comment)?;
        // stream.write_value(self.file_comment)?;
        Ok(stream)
    }
}
impl ValueRead for Directory<Parser> {
    fn read_args<T: Sized>(stream: &mut Stream, _args: &Option<T>) -> Result<Self> {
        let magic: Magic = stream.read_value()?;
        if magic != Magic::Directory {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid directory magic number",
            ));
        }
        let version: u16 = stream.read_value()?;
        let min_version: u16 = stream.read_value()?;
        let flags: u16 = stream.read_value()?;
        let compression_method: CompressionType = stream.read_value()?;
        let last_modification_time: u16 = stream.read_value()?;
        let last_modification_date: u16 = stream.read_value()?;
        let crc_32_uncompressed_data: u32 = stream.read_value()?;
        let compressed_size: u32 = stream.read_value()?;
        let uncompressed_size: u32 = stream.read_value()?;
        let file_name_length: u16 = stream.read_value()?;
        let extra_field_length: u16 = stream.read_value()?;
        let file_comment_length: u16 = stream.read_value()?;
        let number_of_starts: u16 = stream.read_value()?;
        let internal_file_attributes: u16 = stream.read_value()?;
        let external_file_attributes: u32 = stream.read_value()?;
        let offset_of_local_file_header: u32 = stream.read_value()?;
        let mut extra_fields: Vec<Extra<Center>> = Vec::new();
        let file_name = stream.read_exact_size(file_name_length as u64)?;
        let file_name =
            String::from_utf8(file_name).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        let compressed = compression_method == CompressionType::Deflate;
        let mut total_bytes = 0;
        if extra_field_length > 0 {
            loop {
                let position = stream.position()?;
                let extra_field: Extra<Center> = stream.read_value_args(&Some(true))?;
                extra_fields.push(extra_field);
                let size = stream.position()? - position;
                total_bytes += size;
                if total_bytes >= extra_field_length as u64 {
                    break;
                }
            }
        }
        let file_comment = stream.read_exact_size(file_comment_length as u64)?;
        stream.pin()?;
        stream.seek(SeekFrom::Start(offset_of_local_file_header as u64))?;
        let mut file: ZipFile<Parser> = stream.read_value()?;
        if file.flags & 0x0008 != 0 {
            file.data_descriptor = Some(DataDescriptor {
                crc32: crc_32_uncompressed_data,
                compressed_size,
                uncompressed_size,
            })
        }
        stream.seek(SeekFrom::Start(file.data_position))?;
        let data_bytes = stream.read_exact_size(compressed_size as u64)?;
        let mut data: Stream = stream.copy_empty()?;
        data.write_all(&data_bytes)?;
        data.seek_start()?;
        stream.un_pin()?;
        Ok(Self {
            r#type: Parser,
            compressed,
            data,
            version,
            min_version,
            flags,
            compression_method,
            last_modification_time,
            last_modification_date,
            crc_32_uncompressed_data,
            compressed_size,
            uncompressed_size,
            file_name_length,
            extra_field_length,
            file_comment_length,
            number_of_starts,
            internal_file_attributes,
            external_file_attributes,
            offset_of_local_file_header,
            file_name,
            extra_fields,
            file_comment,
            file,
        })
    }
}
