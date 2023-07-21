use std::collections::HashMap;
use std::env;

fn show_help () {
    println!("Help: letter_counting [-h] <filepath>");
}

// Count letters from file input and print the distribution.
fn main() {
    println!("distribution of letters");
    let mut file_path : Option::<String> = Option::<String>::None;
    let mut is_first = true;
    for arg in env::args() {
        if is_first {
            // skip file name arg.
            is_first = false;
            continue;
        }
        if arg == "-h" {
            show_help();
            return; // TODO: exit with non-zero code?
        }
        if file_path.is_some() {
            println !("unrecognized argument: {}", arg);
            show_help();
            return; // TODO: exit with non-zero code?
        }
        file_path = Option::<String>::Some(arg);
        continue;
    }

    let file_path = file_path.expect("unexpected None file_path");

    // Try to open file.
    let res = std::fs::read(&file_path);
    let contents : String;
    match res {
        Ok(bytes) => {
            match String::from_utf8(bytes) {
                Ok(val) => {
                    contents = val;
                },
                Err(err) => {
                    println!("Error parsing UTF-8 from file {}: {}", file_path, err);
                    return;
                }
            }
        },
        Err(err) => {
            println!("Error reading file {}: {}", file_path, err);
            return;
        }
    }

    let mut dist = HashMap::<char, i32>::new();
    count_letters (&contents, &mut dist);
    for (c, cnt) in &dist {
        println! ("{}  : {}", c, cnt);
    }
}

fn count_letters (input : &String, dist : &mut HashMap::<char, i32>) {
    for c in input.chars() {
        let count : &mut i32 = dist.entry(c).or_insert(0);
        *count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_count_letters () {
        let input = String::from("foobar é");
        let mut dist = HashMap::<char, i32>::new();
        count_letters (&input, &mut dist);
        assert_eq!(*dist.get(&'x').unwrap_or(&0), 0);
        assert_eq!(*dist.get(&'o').unwrap_or(&0), 2);
        assert_eq!(*dist.get(&'é').unwrap_or(&0), 1);
    }
}
