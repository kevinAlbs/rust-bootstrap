use std::env;
use std::process;
use minigrep::Config;
use minigrep::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config : Config;
    match Config::new (&args) {
        Ok(v) => config = v,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        }
    }
    match run (config) {
        Ok(_) => {},
        Err(e) => {
            println!("Error running: {}", e);
            process::exit(1);
        }
    }
}
