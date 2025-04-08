use std::fmt::format;
use fast_stream::stream::Stream;
use fast_zip::zip::Zip;
use std::fs;
use fast_stream::crc32::CRC32;

fn main() {
    let zip_file = fs::File::open("./data/h.zip").unwrap();
    let stream = Stream::new(zip_file.into());
    let mut zip = Zip::new(stream);
    let mut data = Stream::new("hello hhhhh".as_bytes().to_vec().into());
    zip.parse().unwrap();

    zip.add_directory(&mut data, "Payload/hello.txt").unwrap();

    let mut output = Stream::new(vec![].into());
    zip.write(&mut output).unwrap();

    // println!("{:?}", data);
    fs::write("./data/output/copy.zip", output.take_data().unwrap()).unwrap();
}
