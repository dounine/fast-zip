use crate::directory::{CompressionMethod, Directory};
use crate::extra::{Center, Extra};
use crate::zip::{Cache, Parser};
use crate::zip_file::ZipFile;
use fast_stream::bytes::{Bytes, StreamSized, ValueRead, ValueWrite};
use fast_stream::endian::Endian;
use fast_stream::stream::Stream;
use std::io::{Read, Result};

impl Directory<Cache> {
    pub fn to_parser(self) -> Directory<Parser> {
        Directory {
            r#type: Parser,
            data: self.data,
            compressed: self.compressed,
            created_zip_spec: self.created_zip_spec,
            created_os: self.created_os,
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
            file_comment_length: self.file_comment_length,
            number_of_starts: self.number_of_starts,
            internal_file_attributes: self.internal_file_attributes,
            external_file_attributes: self.external_file_attributes,
            offset_of_local_file_header: self.offset_of_local_file_header,
            file_name: self.file_name,
            extra_fields: self.extra_fields,
            file_comment: self.file_comment,
            file: self.file.to_parser(),
        }
    }
}
impl Directory<Parser> {
    pub fn into_cache(self) -> Directory<Cache> {
        Directory {
            r#type: Cache,
            data: self.data,
            compressed: self.compressed,
            created_zip_spec: self.created_zip_spec,
            created_os: self.created_os,
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
            file_comment_length: self.file_comment_length,
            number_of_starts: self.number_of_starts,
            internal_file_attributes: self.internal_file_attributes,
            external_file_attributes: self.external_file_attributes,
            offset_of_local_file_header: self.offset_of_local_file_header,
            file_name: self.file_name,
            extra_fields: self.extra_fields,
            file_comment: self.file_comment,
            file: self.file.into_cache(),
        }
    }
}

impl ValueWrite for Directory<Cache> {
    fn write_args<T: Sized>(mut self, endian: &Endian, _args: &Option<T>) -> Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        self.data.seek_start()?;
        stream.write_value(self.data.length())?;
        stream.append(&mut self.data)?;
        stream.write_value(self.compressed)?;
        stream.write_value(self.created_zip_spec)?;
        stream.write_value(self.created_os)?;
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
        stream.write_value(self.file_comment_length)?;
        stream.write_value(self.number_of_starts)?;
        stream.write_value(self.internal_file_attributes)?;
        stream.write_value(self.external_file_attributes)?;
        stream.write_value(self.offset_of_local_file_header)?;
        stream.write_value(self.file_name)?;
        stream.write_value(self.extra_fields)?;
        stream.write_value(self.file_comment)?;
        stream.write_value(self.file)?;
        Ok(stream)
    }
}
impl ValueRead for Directory<Cache> {
    fn read_args<T: StreamSized>(stream: &mut Stream, args: &Option<T>) -> Result<Self> {
        let data: Vec<u8> = stream.read_value()?;
        let compressed: bool = stream.read_value()?;
        let created_zip_spec: u8 = stream.read_value()?;
        let created_os: u8 = stream.read_value()?;
        let extract_zip_spec: u8 = stream.read_value()?;
        let extract_os: u8 = stream.read_value()?;
        let flags: u16 = stream.read_value()?;
        let compression_method: CompressionMethod = stream.read_value()?;
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
        let file_name: String = stream.read_value()?;
        let extra_fields: Vec<Extra<Center>> = stream.read_value()?;
        let file_comment: Vec<u8> = stream.read_value()?;
        let file: ZipFile<Cache> = stream.read_value()?;
        Ok(Self {
            r#type: Cache,
            data: data.into(),
            compressed,
            created_zip_spec,
            created_os,
            extract_zip_spec,
            extract_os,
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
