use fast_stream::stream::Stream;
use fast_zip::zip::Zip;
use std::fs;

fn main() {
    let zip_file = fs::File::open("./data/app.ipa").unwrap();
    let stream = Stream::new(zip_file.into());
    let mut zip = Zip::new(stream);
    let data = Stream::new("hello hhhhh".as_bytes().to_vec().into());
    zip.parse().unwrap();

    zip.add_directory(data, "Payload/hello.txt").unwrap();

    let mut output = Stream::new(vec![].into());
    zip.write(&mut output).unwrap();

    // println!("{:?}", data);
    fs::write("./data/output/copy.zip", output.take_data().unwrap()).unwrap();
}
