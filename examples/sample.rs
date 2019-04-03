extern crate heif;

use heif::reader::HeifReader;
use std::env;

fn main() {
    for arg in env::args().skip(1) {
        println!("\n[{}]\n", arg);
        let mut reader = HeifReader::default();
        reader.load(arg.as_str()).unwrap();
    }
}
