extern crate heif;

fn main() {
  heif::load("./examples/sample.heic").unwrap();
}