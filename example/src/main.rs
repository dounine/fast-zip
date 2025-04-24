use fast_stream::stream::Stream;
use fast_zip::zip::Zip;
use std::fs;

fn main() {
    let zip_file = fs::File::open("./data/hello.zip").unwrap();
    let data = fs::read("./data/hello.zip").unwrap();
    let stream = Stream::new(data.into());
    let mut zip = Zip::new(stream);
    zip.parse().unwrap();
    // let data = Stream::new("hello".as_bytes().to_vec().into());
    // zip.add_directory(data, "b.txt").unwrap();

    let mut output = Stream::new(vec![].into());
    zip.write(&mut output).unwrap();

    fs::write("./data/copy.zip", output.take_data().unwrap()).unwrap();
}
