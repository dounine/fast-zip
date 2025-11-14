use crate::eocd::EoCd;
use crate::zip::{Cache, Parser};
use fast_stream::bytes::{Bytes, ValueRead, ValueWrite};
use fast_stream::endian::Endian;
use fast_stream::stream::Stream;
impl EoCd<Parser> {
    pub fn to_cache(self) -> EoCd<Cache> {
        EoCd {
            r#type: Cache,
            number_of_disk: self.number_of_disk,
            directory_starts: self.directory_starts,
            number_of_directory_disk: self.number_of_directory_disk,
            entries: self.entries,
            size: self.size,
            offset: self.offset,
            comment_length: self.comment_length,
        }
    }
}
impl EoCd<Cache> {
    pub fn to_parser(self) -> EoCd<Parser> {
        EoCd {
            r#type: Parser,
            number_of_disk: self.number_of_disk,
            directory_starts: self.directory_starts,
            number_of_directory_disk: self.number_of_directory_disk,
            entries: self.entries,
            size: self.size,
            offset: self.offset,
            comment_length: self.comment_length,
        }
    }
}
impl ValueRead for EoCd<Cache> {
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
        Ok(Self {
            r#type: Cache,
            number_of_disk: stream.read_value()?,
            directory_starts: stream.read_value()?,
            number_of_directory_disk: stream.read_value()?,
            entries: stream.read_value()?,
            size: stream.read_value()?,
            offset: stream.read_value()?,
            comment_length: stream.read_value()?,
        })
    }
}
impl ValueWrite for EoCd<Cache> {
    fn write(self, endian: &Endian) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(self.number_of_disk)?;
        stream.write_value(self.directory_starts)?;
        stream.write_value(self.number_of_directory_disk)?;
        stream.write_value(self.entries)?;
        stream.write_value(self.size)?;
        stream.write_value(self.offset)?;
        stream.write_value(self.comment_length)?;
        Ok(stream)
    }
}
