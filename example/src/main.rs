use std::fs;
use fast_stream::stream::Stream;
use fast_zip::zip::Zip;

fn main() {
    let zip_file = fs::File::open("./data/hello.zip").unwrap();
    let stream = Stream::new(zip_file.into());
    let mut zip = Zip::new(stream);
    zip.parse().unwrap();

    let mut output = Stream::new(vec![].into());
    zip.write(&mut output).unwrap();

    // println!("{:?}", data);
    fs::write("./data/copy.zip", output.take_data().unwrap()).unwrap();
}
