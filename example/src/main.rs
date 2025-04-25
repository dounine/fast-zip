use fast_stream::stream::Stream;
use fast_zip::zip::Zip;
use std::fs;

fn main() {
    let data = fs::read("./data/hello.zip").unwrap();
    let stream = Stream::new(data.into());
    let mut zip = Zip::new(stream);
    zip.parse().unwrap();
    let data = Stream::new("hello hello world".as_bytes().to_vec().into());
    zip.add_directory(data, "hello2.txt").unwrap();

    let mut output = Stream::new(vec![].into());
    zip.write(&mut output).unwrap();

    fs::write("./data/copy.zip", output.take_data().unwrap()).unwrap();
}
