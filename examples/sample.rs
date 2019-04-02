extern crate heif;

use std::env;
use heif::reader::HeifReader;

fn main() {
    for arg in env::args().skip(1) {
        println!("\n[{}]\n", arg);
        let mut reader = HeifReader::default();
        reader.load(arg.as_str()).unwrap();
        println!("{:?}", reader);
    }
}
