use std::fs;
use std::env;

pub struct Config<'a> {
    query : &'a str,
    file_path : &'a str,
    ignore_case: bool
}

impl<'a> Config<'a> {
    pub fn new (args: &[String]) -> Result<Config, String> {
        if args.len() < 3 {
            return Result::Err(format!("expected at least 2 arguments <query> <file_path> [options], got {}", args.len() - 1))
        }
        let query = &args[1];
        let file_path = &args[2];
        let mut ignore_case = env::var("IGNORE_CASE").is_ok();
        if args.len() > 3 {
            for arg in &args[2..] {
                if arg == "--ignore-case" {
                    ignore_case = true;
                } else {
                    println !("Unrecognized arg: {}", arg);
                }
            }
        }
        return Result::Ok(Config{query, file_path, ignore_case})
    }
}

use std::error::Error;

// Q: is the `::` in type declarations optional sometimes?
// `pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {` also compiles.
pub fn search<'a>(query: &str, contents: &'a str) -> Vec::<&'a str> {
    let mut results = Vec::<&str>::new();
    let lines = contents.lines();
    for line in lines {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}

pub fn search_insensitive<'a>(query: &str, contents: &'a str) -> Vec::<&'a str> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::<&str>::new();
    let lines = contents.lines();
    for line in lines {
        let line_lower = line.to_lowercase();
        // Q: why is this a compiler error: `if line_lower.contains(query_lower.as_str()) {`?
        // With error: 
        //         error[E0277]: expected a `FnMut<(char,)>` closure, found `String`
        //         |
        //    40   |         if line_lower.contains(query_lower) {
        //         |                       -------- ^^^^^^^^^^^ the trait `Pattern<'_>` is not implemented for `String`
        //         |                       |
        //         |                       required by a bound introduced by this call
        // String implements the Pattern trait.
        // A: Appears `&query_lower` fixes. I do not understand why.
        if line_lower.contains(&query_lower) {
            results.push(line);
        }
    }
    results
}

pub fn run(config : Config) -> Result<(), Box<dyn Error>> {
    let contents : String;
    match fs::read_to_string(config.file_path) {
        Ok(v) => contents = v,
        Err(e) => return Result::Err(Box::new(e))
    };
    let results;
    if config.ignore_case {
        results = search_insensitive(config.query, contents.as_str());
    } else {
        results = search(config.query, contents.as_str());
    }
    for line in results {
        println!("{}", line);
    }
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "FOO";
        let contents = "\
        foo
        bar
        baz";
        assert_eq!(vec!["foo"], search_insensitive(query, contents));
    }
}