extern crate heif;

use std::env;

fn main() {
    for arg in env::args().skip(1) {
        println!("\n[{}]\n", arg);
        heif::reader::HeifReader::from(arg.as_str()).unwrap();
    }
}
