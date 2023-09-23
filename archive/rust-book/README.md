Q: What is the `cannot move out of <var> which is behind a mutable reference` error?
A: Example:

```rust
struct MyStruct {
    v : i32,
}

// calling `takes_value` results in a move.
fn takes_value (x : MyStruct) {
    println !("takes_value: {}", x.v);
}

fn takes_mutable_ref (x : &mut MyStruct) {
    println !("takes_mutable_ref: {}", x.v);
    takes_value (*x); // Results in `cannot move out of `*x` which is behind a mutable reference`
}

fn main() {
    let mut x = MyStruct{v: 123};
    takes_mutable_ref (&mut x);
    println !("main: {}", x.v);
}
```

// Q: How to get stdout for test?
// A: cargo test -- --show-output
--
Macros are invoked with !
--
Access book with: `rustup doc --book`.