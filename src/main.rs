extern crate mongo_driver;
use mongo_driver::client::Uri;

fn main() {
    println!("Hello, world!");
    let uri = Uri::new("mongodb://localhost:27017/").unwrap();
}
