use std::io;
fn main() {
    let mut input = String::new();
    io::stdin().read_line (&mut input).expect("failed to read line");
    input = input.trim().to_string();
    let input : u64 = input.parse::<u64>().expect("failed to parse int64");
    println!("got input: {input}");
    let mut prev = 0;
    let mut curr = 1;
    let out;
    if input == 0 {
        out = 0;
    } else if input == 1 {
        out = 1;
    } else {
        for _ in 0..input {
            let next = curr + prev;
            prev = curr;
            curr = next;
        }
        out = curr
    }
    println!("answer is {out}");
}
