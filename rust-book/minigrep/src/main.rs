use std::env;
use std::fs;

fn parse_config (args : &[String]) -> (&str, &str) {
    let query = &args[1];
    let file_path = &args[2];
    (query, file_path)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (query, file_path) = parse_config (&args);
    println! ("Searching for {} in file {}", query, file_path);
    let contents : String = fs::read_to_string(file_path).expect("Should have been able to read file");
    println!("With text:\n{contents}");
}
