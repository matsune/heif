extern crate heif;

use heif::reader::HeifReader;
use std::env;

fn main() {
    for arg in env::args().skip(1) {
        println!("\n[{}]\n", arg);
        let mut reader = HeifReader::default();
        reader.load(arg.as_str()).unwrap();
        let grid_item_ids = reader.get_item_list_by_type("grid".parse().unwrap());
        if grid_item_ids.is_empty() {
            return;
        }
        let grid = reader.get_grid_item(grid_item_ids[0]).unwrap();
        println!("grid {:?}", grid);
        let master_image_ids = reader.get_master_image_ids();
        if master_image_ids.is_empty() {
            return;
        }
        println!("master_image_ids {:?}", master_image_ids);
        let first_tile_id = master_image_ids[0];
    }
}
