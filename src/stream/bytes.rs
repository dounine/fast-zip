use crate::stream::endian::Endian;
use crate::stream::stream::Stream;
use std::io;
use std::io::{Read, Seek, Write};

#[allow(dead_code)]
pub trait ValueRead<T: Read + Write + Seek>: Sized {
    fn read(stream: &mut Stream<T>) -> io::Result<Self>;
}
#[allow(dead_code)]
impl<T: Read + Write + Seek> Stream<T> {
    pub fn read_value<Value: ValueRead<T>>(&mut self) -> io::Result<Value> {
        Value::read(self)
    }
    pub fn read_size(&mut self, size: u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; size as usize];
        self.inner.read_exact(&mut buf)?;
        Ok(buf)
    }
}
#[macro_export]
macro_rules! value_read {
    ($($typ:ty, $size:expr),*) => {
        $(
            impl<T: std::io::Read + std::io::Write + std::io::Seek> ValueRead<T> for $typ {
                fn read(stream: &mut Stream<T>) -> std::io::Result<Self> {
                    use crate::stream::endian::Endian;
                    let mut buf = [0u8; $size];
                    stream.read_exact(&mut buf)?;
                    let value = match stream.endian {
                        Endian::Big => <$typ>::from_be_bytes(buf),
                        Endian::Little => <$typ>::from_le_bytes(buf),
                    };
                    Ok(value)
                }
            }
        )*
    }
}
value_read!(u8, 1, u16, 2, u32, 4, u64, 8);

impl<T: Read + Write + Seek> ValueRead<T> for [u8; 4] {
    fn read(stream: &mut Stream<T>) -> io::Result<Self> {
        let mut value = [0u8; 4];
        stream.read_exact(&mut value)?;
        Ok(value)
    }
}
