# TIL

## TIL 1: All references have lifetimes.
```rust
// All references have lifetimes.
// Sometimes lifetime specifiers can be elided.
// See: https://stackoverflow.com/a/72778738

// Example: No lifetime specifier needed.
fn foo(input: &str) -> &str {
    return &input[1..];
}

// Example: Equivalent to above.
fn foo2<'a>(input: &'a str) -> &'a str {
    return &input[1..];
}

#[test]
fn test_foos() {
    assert_eq!(foo("abc"), "bc");
    assert_eq!(foo2("abc"), "bc");
}
```

# Goofs
## Goof 1: Cannot construct a Path from temporary strings.

```rust
// Path does not own the underlying string:
let path = {
    let path_from_env = std::env::var("KMS_PROVIDERS_PATH");
    match path_from_env {
        Ok(path_str) => Path::new(&path_str), // Error: `path_str` does not live long enough.
        Err(_) => {
            panic!("Unable to read path");
        }
    }
};
// Use the owned PathBuf instead:
let path = {
    let path_from_env = std::env::var("KMS_PROVIDERS_PATH");
    match path_from_env {
        Ok(path_str) => PathBuf::from(path_str),
        Err(_) => {
            panic!("Unable to read path");
        }
    }
};
```

# Rust Q&A

## Q4: Does Rust support socketTimeoutMS?
A: No. See RUST-2321.

## Q3: How to get stdout for test?
A: cargo test -- --show-output

## Q2: How do I return an Error from multiple types?
A: One option: use a result with a boxed `dyn Error`:
`std::result::Result<T, Box<dyn std::error::Error>>`

## Q1: What happens when `as` is used with out-of-range value?
A: Casting f64 to i32 appears to cap value:
```rust
let as_f64 = 4294967297.0;
let as_i32 = as_f64 as i32;
println! ("as_i32={:?}, as_f64={:?}", as_i32, as_f64);
// Prints:
// as_i32=2147483647, as_f64=4294967297.0
```

# Rust Driver Q&A

## Q4: What is the default maxPoolSize?

`10`. This differs from the specification.

## Q3: Is it preferable to reference `bson` crate directly, or through `mongodb`?
Example:
```rust
use bson::Document;
// vs.
use mongodb::bson::Document;
```
A: (Open)

## Q2: Does `Cursor.try_next` return None if `getMore` is still in flight?
A: No? Blocking `getMore` with a failpoint results in this loop returning all documents:
```rust
while let Some(book) = cursor.try_next().await? {
    cnt += 1;
}
```

## Q1: Is `collection.find()` lazy?
A: No. `collection.find()` sends the "find" command on invocation. This differs from C driver. C driver does not send "find" command until the first iteration of the returned cursor.

