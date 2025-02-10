use crate::magic::Magic;
use derive::NumToEnum;
use fast_stream::bytes::{ValueRead, ValueWrite};
use fast_stream::endian::Endian;
use fast_stream::enum_to_bytes;
use fast_stream::pin::Pin;
use fast_stream::stream::Stream;
use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Write};

#[repr(u16)]
#[derive(Debug, Clone, Default, NumToEnum)]
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
enum_to_bytes!(CompressionType, u16);
const ZIP_FILE_HEADER_SIZE: usize = size_of::<Magic>()
    + size_of::<u16>() * 2
    + size_of::<CompressionType>()
    + size_of::<u16>() * 2
    + size_of::<u32>() * 3
    + size_of::<u16>() * 2;
#[derive(Debug, Default)]
pub struct ZipFile {
    pub min_version: u16,
    pub bit_flag: u16,
    pub compression_method: CompressionType,
    pub last_modification_time: u16,
    pub last_modification_date: u16,
    pub crc_32_uncompressed_data: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_name: String,
    pub extra_field: Vec<u8>,
    pub data_position: u64,
}
impl ValueWrite for ZipFile {
    fn write(&self, endian: &Endian) -> std::io::Result<Vec<u8>> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(&Magic::File)?;
        stream.write_value(&self.min_version)?;
        stream.write_value(&self.bit_flag)?;
        stream.write_value(&self.compression_method)?;
        stream.write_value(&self.last_modification_time)?;
        stream.write_value(&self.last_modification_date)?;
        stream.write_value(&self.crc_32_uncompressed_data)?;
        stream.write_value(&self.compressed_size)?;
        stream.write_value(&self.uncompressed_size)?;
        stream.write_value(&self.file_name_length)?;
        stream.write_value(&self.extra_field_length)?;
        stream.write_value(&self.file_name)?;
        // stream.write_value(&self.extra_field)?;
        Ok(stream.take_data())
    }
}
impl ZipFile {
    pub fn size(&self) -> usize {
        // size_of::<Magic>()
        //     + size_of_val(&self.min_version)
        //     + size_of_val(&self.bit_flag)
        //     + size_of_val(&self.compression_method)
        //     + size_of_val(&self.last_modification_time)
        //     + size_of_val(&self.last_modification_date)
        //     + size_of_val(&self.crc_32_uncompressed_data)
        //     + size_of_val(&self.compressed_size)
        //     + size_of_val(&self.uncompressed_size)
        //     + size_of_val(&self.file_name_length)
        //     + size_of_val(&self.extra_field_length)
        ZIP_FILE_HEADER_SIZE
            + self.file_name.as_bytes().len()
            + self.extra_field.len()
            + self.compressed_size as usize
    }
}
impl ZipFile {
    pub fn origin_data<T: Read + Write + Seek>(
        &self,
        stream: &mut Stream<T>,
    ) -> std::io::Result<Vec<u8>> {
        stream.pin()?;
        stream.seek(SeekFrom::Start(self.data_position))?;
        let data = stream.read_size(self.compressed_size as u64)?;
        stream.un_pin()?;
        Ok(data)
    }
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
            data_position: 0,
        };
        let file_name = stream.read_size(file.file_name_length as u64)?;
        let file_name =
            String::from_utf8(file_name).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        file.file_name = file_name.clone();
        file.extra_field = stream.read_size(file.extra_field_length as u64)?;
        file.data_position = stream.stream_position()?;
        // let data = file.un_compressed_data(stream)?;
        Ok(file)
    }
}
const DIRECTORY_HEADER_SIZE: usize = size_of::<Magic>()
    + size_of::<u16>() * 6
    + size_of::<u32>() * 3
    + size_of::<u16>() * 5
    + size_of::<u32>() * 2;
#[derive(Debug)]
pub struct Directory {
    pub version: u16,
    pub min_version: u16,
    pub bit_flag: u16,
    pub compression_method: u16,
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
    pub extra_field: Vec<u8>,
    pub file_comment: Vec<u8>,
    pub file: ZipFile,
}
impl Directory {
    pub fn size(&self) -> usize {
        DIRECTORY_HEADER_SIZE
            + self.file_name.as_bytes().len()
            + self.extra_field.len()
            + self.file_comment.len()
    }
}
impl ValueWrite for Directory {
    fn write(&self, endian: &Endian) -> std::io::Result<Vec<u8>> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(&Magic::Directory)?;
        stream.write_value(&self.version)?;
        stream.write_value(&self.min_version)?;
        stream.write_value(&self.bit_flag)?;
        stream.write_value(&self.compression_method)?;
        stream.write_value(&self.last_modification_time)?;
        stream.write_value(&self.last_modification_date)?;
        stream.write_value(&self.crc_32_uncompressed_data)?;
        stream.write_value(&self.compressed_size)?;
        stream.write_value(&self.uncompressed_size)?;
        stream.write_value(&self.file_name_length)?;
        stream.write_value(&self.extra_field_length)?;
        stream.write_value(&self.file_comment_length)?;
        stream.write_value(&self.number_of_starts)?;
        stream.write_value(&self.internal_file_attributes)?;
        stream.write_value(&self.external_file_attributes)?;
        stream.write_value(&self.offset_of_local_file_header)?;
        stream.write_value(&self.file_name)?;
        stream.write_value(&self.extra_field)?;
        stream.write_value(&self.file_comment)?;
        Ok(stream.take_data())
    }
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
        stream.un_pin()?;
        info.file = file;
        Ok(info)
    }
}
