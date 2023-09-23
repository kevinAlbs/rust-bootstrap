use std::collections::HashMap;

fn main() {
    let args = std::env::args();
    let mut list = Vec::new();
    for arg in args.skip(1) {
        let guess_res = arg.parse::<i32>();
        match guess_res {
            Ok(value) => {
                list.push(value);
            },
            Err(e) => {
                println !("Expected integer argument, got: {arg}. Error: {}",                 e.to_string());
                return;
            }
        }
    }
    list.sort();
    println !("Median is: {}", list.get(list.len() / 2).expect("Unexpected OOB"));

    let mut map = HashMap::new();
    for n in list {
        let e = map.entry(n).or_insert(0);
        *e += 1;
    }

    let mut max_count = -1;
    let mut max_value = 0;
    for (key, value) in map {
        if key >= max_count {
            max_count = value;
            max_value = key;
        }
    }

    println !("Mode is: {}", max_value);
}
