use crate::directory::Directory;
use crate::eocd::EoCd;
use crate::zip::{Cache, CompressionLevelWrapper, Parser, Zip};
use fast_stream::bytes::{Bytes, ValueRead, ValueWrite};
use fast_stream::endian::Endian;
use fast_stream::stream::Stream;
use indexmap::IndexMap;
use std::time::Instant;

impl Zip<Parser> {
    pub fn into_cache(self) -> Zip<Cache> {
        let mut directories = IndexMap::new();
        for (k, v) in self.directories {
            directories.insert(k, v.into_cache());
        }
        Zip {
            stream_size: self.stream_size,
            stream: self.stream,
            crc32_computer: self.crc32_computer,
            eo_cd: self.eo_cd.map(|e| e.to_cache()),
            write_clear: self.write_clear,
            compression_level: self.compression_level,
            directories,
        }
    }
}
impl Zip<Cache> {
    pub fn into_bytes(self) -> std::io::Result<Vec<u8>> {
        let mut bytes = Stream::empty();
        bytes.write_value(self)?;
        bytes.take_data()
    }
    pub fn from_cache(data: Vec<u8>) -> std::io::Result<Self> {
        Stream::new(data.into()).read_value()
    }
    pub fn into_parser(self) -> Zip<Parser> {
        let mut directories = IndexMap::new();
        for (k, v) in self.directories {
            directories.insert(k, v.to_parser());
        }
        Zip {
            stream_size: self.stream_size,
            stream: self.stream,
            crc32_computer: self.crc32_computer,
            eo_cd: self.eo_cd.map(|e| e.to_parser()),
            write_clear: self.write_clear,
            compression_level: self.compression_level,
            directories,
        }
    }
}
impl ValueRead for Zip<Cache> {
    fn read(stream: &mut Stream) -> std::io::Result<Self> {
        let stream_size: u64 = stream.read_value()?;
        let data: Option<Stream> = if stream.read_value::<bool>()? {
            let len: u64 = stream.read_value()?;
            Some(stream.copy_size(len as usize)?)
        } else {
            None
        };
        let crc32_computer: bool = stream.read_value()?;
        let eo_cd: Option<EoCd<Cache>> = if stream.read_value::<bool>()? {
            Some(stream.read_value()?)
        } else {
            None
        };
        let write_clear: bool = stream.read_value()?;
        let compression_level: CompressionLevelWrapper = stream.read_value()?;
        let directories_len: u64 = stream.read_value()?;
        let mut directories = IndexMap::with_capacity(directories_len as usize);
        for _ in 0..directories_len {
            let k: String = stream.read_value()?;
            let v: Directory<Cache> = stream.read_value()?;
            directories.insert(k, v);
        }
        Ok(Self {
            stream_size,
            stream: data,
            crc32_computer,
            eo_cd,
            write_clear,
            compression_level: compression_level.0,
            directories,
        })
    }
}
impl ValueWrite for Zip<Cache> {
    fn write(self, endian: &Endian) -> std::io::Result<Stream> {
        let mut stream = Stream::empty();
        stream.with_endian(endian.clone());
        stream.write_value(self.stream_size)?;
        stream.write_value(self.stream.is_some())?;
        if let Some(mut data) = self.stream {
            stream.write_value(data.length())?;
            stream.append(&mut data)?;
        }
        stream.write_value(self.crc32_computer)?;
        stream.write_value(self.eo_cd.is_some())?;
        if let Some(eo_cd) = self.eo_cd {
            stream.write_value(eo_cd)?;
        }
        stream.write_value(self.write_clear)?;
        stream.write_value(CompressionLevelWrapper(self.compression_level))?;
        stream.write_value(self.directories.len() as u64)?;
        let stream_time = Instant::now();
        for (k, v) in self.directories {
            stream.write_value(k)?;
            stream.write_value(v)?;
        }
        dbg!(stream_time.elapsed());
        Ok(stream)
    }
}
