use std::fs;
use std::io;

#[test]
fn readFile() {
    let mut data = fs::read_to_string("custom.json");
    println!("{:?}", data);
}
