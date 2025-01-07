use crate::magic::Magic;
use derive::NumToEnum;
use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use fast_stream::bytes::ValueRead;
use fast_stream::pin::Pin;
use fast_stream::stream::Stream;

#[repr(u16)]
#[derive(Debug, Default, NumToEnum)]
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
impl<T: Read + Write + Seek> ValueRead<T> for CompressionType {
    fn read(stream: &mut Stream<T>) -> std::io::Result<Self> {
        let value: u16 = stream.read_value()?;
        Ok(value.into())
    }
}

#[derive(Debug, Default)]
pub struct ZipFile {
    min_version: u16,
    bit_flag: u16,
    compression_method: CompressionType,
    last_modification_time: u16,
    last_modification_date: u16,
    crc_32_uncompressed_data: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    extra_field_length: u16,
    file_name: String,
    extra_field: Vec<u8>,
}
impl ZipFile {
    pub fn un_compressed_data<T: Read + Write + Seek>(
        &self,
        stream: &mut Stream<T>,
    ) -> std::io::Result<Vec<u8>> {
        stream.pin()?;
        let compressed_data = stream.read_size(self.compressed_size as u64)?;
        let data = if self.uncompressed_size != self.compressed_size {
            let uncompress_data = decompress_to_vec(&compressed_data)
                .map_err(|e| Error::new(ErrorKind::InvalidData, std::fmt::Error::default()))?;
            uncompress_data
        } else {
            compressed_data
        };
        stream.un_pin()?;
        Ok(data)
    }
}

impl<T: Read + Write + Seek> ValueRead<T> for ZipFile {
    fn read(stream: &mut Stream<T>) -> std::io::Result<Self> {
        let magic: Magic = stream.read_value()?;
        if magic != Magic::File {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid directory file magic number",
            ));
        }
        let mut file = ZipFile {
            min_version: stream.read_value()?,
            bit_flag: stream.read_value()?,
            compression_method: stream.read_value()?,
            last_modification_time: stream.read_value()?,
            last_modification_date: stream.read_value()?,
            crc_32_uncompressed_data: stream.read_value()?,
            compressed_size: stream.read_value()?,
            uncompressed_size: stream.read_value()?,
            file_name_length: stream.read_value()?,
            extra_field_length: stream.read_value()?,
            file_name: "".to_string(),
            extra_field: vec![],
        };
        let file_name = stream.read_size(file.file_name_length as u64)?;
        let file_name =
            String::from_utf8(file_name).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        file.file_name = file_name.clone();
        file.extra_field = stream.read_size(file.extra_field_length as u64)?;
        // let data = file.un_compressed_data(stream)?;
        Ok(file)
    }
}

#[derive(Debug)]
pub struct Directory {
    version: u16,
    min_version: u16,
    bit_flag: u16,
    compression_method: u16,
    last_modification_time: u16,
    last_modification_date: u16,
    crc_32_uncompressed_data: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    extra_field_length: u16,
    file_comment_length: u16,
    number_of_starts: u16,
    internal_file_attributes: u16,
    external_file_attributes: u32,
    offset_of_local_file_header: u32,
    file_name: String,
    extra_field: Vec<u8>,
    file_comment: Vec<u8>,
    file: ZipFile,
}
impl<T: Read + Write + Seek> ValueRead<T> for Directory {
    fn read(stream: &mut Stream<T>) -> std::io::Result<Self> {
        let magic: Magic = stream.read_value()?;
        if magic != Magic::Directory {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid directory magic number",
            ));
        }
        let mut file = ZipFile::default();
        let mut info = Self {
            version: stream.read_value()?,
            min_version: stream.read_value()?,
            bit_flag: stream.read_value()?,
            compression_method: stream.read_value()?,
            last_modification_time: stream.read_value()?,
            last_modification_date: stream.read_value()?,
            crc_32_uncompressed_data: stream.read_value()?,
            compressed_size: stream.read_value()?,
            uncompressed_size: stream.read_value()?,
            file_name_length: stream.read_value()?,
            extra_field_length: stream.read_value()?,
            file_comment_length: stream.read_value()?,
            number_of_starts: stream.read_value()?,
            internal_file_attributes: stream.read_value()?,
            external_file_attributes: stream.read_value()?,
            offset_of_local_file_header: stream.read_value()?,
            file_name: "".to_string(),
            extra_field: vec![],
            file_comment: vec![],
            file,
        };
        let file_name = stream.read_size(info.file_name_length as u64)?;
        let file_name =
            String::from_utf8(file_name).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        info.file_name = file_name;
        info.extra_field = stream.read_size(info.extra_field_length as u64)?;
        info.file_comment = stream.read_size(info.file_comment_length as u64)?;
        stream.pin()?;
        stream.seek(SeekFrom::Start(info.offset_of_local_file_header as u64))?;
        let file: ZipFile = stream.read_value()?;
        dbg!(file);
        stream.un_pin()?;
        Ok(info)
    }
}
