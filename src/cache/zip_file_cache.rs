use crate::zip::{Cache, Parser};
use crate::zip_file::ZipFile;
use fast_stream::bytes::{Bytes, ValueRead, ValueWrite};
use fast_stream::endian::Endian;
use fast_stream::stream::Stream;
use std::io::Result;
impl ZipFile<Parser> {
    pub fn into_cache(self) -> ZipFile<Cache> {
        ZipFile {
            r#type: Cache,
            extract_zip_spec: self.extract_zip_spec,
            extract_os: self.extract_os,
            flags: self.flags,
            compression_method: self.compression_method,
            last_modification_time: self.last_modification_time,
            last_modification_date: self.last_modification_date,
            crc_32_uncompressed_data: self.crc_32_uncompressed_data,
            compressed_size: self.compressed_size,
            uncompressed_size: self.uncompressed_size,
            file_name_length: self.file_name_length,
            extra_field_length: self.extra_field_length,
            file_name: self.file_name,
            extra_fields: self.extra_fields,
            data_descriptor: self.data_descriptor,
            data_position: self.data_position,
        }
    }
}
impl ZipFile<Cache> {
    pub fn to_parser(self) -> ZipFile<Parser> {
        ZipFile {
            r#type: Parser,
            extract_zip_spec: self.extract_zip_spec,
            extract_os: self.extract_os,
            flags: self.flags,
            compression_method: self.compression_method,
            last_modification_time: self.last_modification_time,
            last_modification_date: self.last_modification_date,
            crc_32_uncompressed_data: self.crc_32_uncompressed_data,
            compressed_size: self.compressed_size,
            uncompressed_size: self.uncompressed_size,
            file_name_length: self.file_name_length,
            extra_field_length: self.extra_field_length,
            file_name: self.file_name,
            extra_fields: self.extra_fields,
            data_descriptor: self.data_descriptor,
            data_position: self.data_position,
        }
    }
}
impl ValueRead for ZipFile<Cache> {
    fn read(stream: &mut Stream) -> Result<Self> {
        let file = ZipFile {
            r#type: Cache,
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
            file_name: stream.read_value()?,
            extra_fields: stream.read_value()?,
            data_descriptor: stream.read_value()?,
            data_position: stream.read_value()?,
        };
        Ok(file)
    }
}
impl ValueWrite for ZipFile<Cache> {
    fn write(self, endian: &Endian) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(self.extract_zip_spec)?;
        stream.write_value(self.extract_os)?;
        stream.write_value(self.flags)?;
        stream.write_value(self.compression_method)?;
        stream.write_value(self.last_modification_time)?;
        stream.write_value(self.last_modification_date)?;
        stream.write_value(self.crc_32_uncompressed_data)?;
        stream.write_value(self.compressed_size)?;
        stream.write_value(self.uncompressed_size)?;
        stream.write_value(self.file_name_length)?;
        stream.write_value(self.extra_field_length)?;
        stream.write_value(self.file_name)?;
        stream.write_value(self.extra_fields)?;
        stream.write_value(self.data_descriptor)?;
        stream.write_value(self.data_position)?;
        Ok(stream)
    }
}
