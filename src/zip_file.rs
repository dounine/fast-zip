use crate::directory::CompressionMethod;
use crate::extra::{Extra};
use crate::magic::Magic;
use crate::zip::Parser;
use fast_stream::bytes::{Bytes, StreamSized, ValueRead, ValueWrite};
use fast_stream::endian::Endian;
use fast_stream::pin::Pin;
use fast_stream::stream::Stream;
use std::io::{Error, ErrorKind, Result, Seek, Write};

const ZIP_FILE_HEADER_SIZE: usize = Magic::byte_size()
    + size_of::<u16>() * 2
    + CompressionMethod::byte_size()
    + size_of::<u16>() * 2
    + size_of::<u32>() * 3
    + size_of::<u16>() * 2;
#[derive(Debug, Clone)]
pub struct DataDescriptor {
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}
impl DataDescriptor {
    pub fn size() -> usize {
        4 * 4
    }
}
impl ValueRead for DataDescriptor {
    fn read_args<T: StreamSized>(stream: &mut Stream, args: &Option<T>) -> Result<Self> {
        let _magic: [u8; 4] = stream.read_value()?;
        Ok(Self {
            crc32: stream.read_value()?,
            compressed_size: stream.read_value()?,
            uncompressed_size: stream.read_value()?,
        })
    }
}
impl ValueWrite for DataDescriptor {
    fn write_args<T: StreamSized>(self, endian: &Endian, args: &Option<T>) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        let magic = &[0x50, 0x4B, 0x07, 0x08];
        stream.write(magic)?;
        stream
            .write_value(self.crc32)?
            .write_value_args(self.compressed_size, args)?
            .write_value_args(self.uncompressed_size, args)?;
        Ok(stream)
    }
}
#[derive(Debug, Clone)]
pub struct ZipFile<TYPE> {
    pub r#type: TYPE,
    pub extract_zip_spec: u8,
    pub extract_os: u8,
    pub flags: u16,
    pub compression_method: CompressionMethod,
    pub last_modification_time: u16,
    pub last_modification_date: u16,
    pub crc_32_uncompressed_data: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_name: String,
    pub extra_fields: Vec<Extra>,
    pub data_descriptor: Option<DataDescriptor>,
    pub data_position: u64,
}
impl ValueWrite for ZipFile<Parser> {
    fn write_args<T: StreamSized>(mut self, endian: &Endian, args: &Option<T>) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value_args(Magic::File, args)?;
        let compression_method = if self.uncompressed_size == 0 {
            CompressionMethod::Store
        } else {
            self.compression_method
        };
        let file_is_dir = self.file_name.ends_with("/");
        if file_is_dir {
            stream.write_value_args(10_u8, args)?; //extract_zip_spec
        } else {
            stream.write_value_args(14_u8, args)?; //extract_zip_spec
        }

        let mut extra_field_stream = Stream::empty();
        self.extra_field_length = 0;
        for extra_field in self.extra_fields {
            self.extra_field_length += extra_field.size();
            extra_field_stream.write_value(extra_field)?;
        }
        extra_field_stream.seek_start()?;

        self.file_name_length = self.file_name.as_bytes().len() as u16;

        stream.write_value_args(self.extract_os, args)?;
        stream.write_value_args(self.flags, args)?;
        stream.write_value_args(compression_method, args)?;
        stream.write_value_args(self.last_modification_time, args)?;
        stream.write_value_args(self.last_modification_date, args)?;
        stream.write_value_args(self.crc_32_uncompressed_data, args)?;
        stream.write_value_args(self.compressed_size, args)?;
        stream.write_value_args(self.uncompressed_size, args)?;
        stream.write_value_args(self.file_name_length, args)?;
        stream.write_value_args(self.extra_field_length, args)?;
        stream.write(self.file_name.as_bytes())?;
        stream.append(&mut extra_field_stream)?;
        if let Some(data_descriptor) = self.data_descriptor {
            stream.write_value_args(data_descriptor, args)?;
        }
        Ok(stream)
    }
}
impl ValueRead for ZipFile<Parser> {
    fn read_args<T: Sized>(stream: &mut Stream, _args: &Option<T>) -> Result<Self> {
        let magic: Magic = stream.read_value()?;
        if magic != Magic::File {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid directory file magic number",
            ));
        }
        let mut file = ZipFile {
            r#type: Parser,
            extract_zip_spec: stream.read_value()?,
            extract_os: stream.read_value()?,
            flags: stream.read_value()?,
            compression_method: stream.read_value()?,
            last_modification_time: stream.read_value()?,
            last_modification_date: stream.read_value()?,
            crc_32_uncompressed_data: stream.read_value()?,
            compressed_size: stream.read_value()?,
            uncompressed_size: stream.read_value()?,
            file_name_length: stream.read_value()?,
            extra_field_length: stream.read_value()?,
            file_name: "".to_string(),
            extra_fields: vec![],
            data_descriptor: None,
            data_position: 0,
        };
        let file_name = stream.read_exact_size(file.file_name_length as u64)?;
        let file_name =
            String::from_utf8(file_name).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        file.file_name = file_name.clone();
        let mut total_bytes = 0;
        if file.extra_field_length > 0 {
            loop {
                let position = stream.position()?;
                let extra_field: Extra = stream.read_value()?; //.read_exact_size(file.extra_field_length as u64)?;
                file.extra_fields.push(extra_field);
                let size = stream.position()? - position;
                total_bytes += size;
                if total_bytes >= file.extra_field_length as u64 {
                    break;
                }
            }
        }
        file.data_position = stream.stream_position()?;
        Ok(file)
    }
}
impl ZipFile<Parser> {
    pub fn size(&self) -> usize {
        let mut bytes = ZIP_FILE_HEADER_SIZE + self.file_name.as_bytes().len();
        for extra_field in &self.extra_fields {
            bytes += extra_field.size() as usize
        }
        let data_descriptor_size = if self.data_descriptor.is_some() {
            DataDescriptor::size()
        } else {
            0
        };
        bytes + data_descriptor_size
    }
}
