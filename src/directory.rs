use crate::stream::bytes::ValueRead;
use crate::stream::endian::Endian;
use crate::stream::stream::Stream;
use std::io::{Error, ErrorKind, Read, Seek, Write};

#[derive(Debug, Default)]
pub struct ZipFile {
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
    file_name: String,
    extra_field: Vec<u8>,
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
    fn read(stream: &mut Stream<T>, _endian: &Endian) -> std::io::Result<Self> {
        let magic: u32 = stream.read_value()?;
        if magic != 0x02014b50 {
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
        Ok(info)
    }
}
