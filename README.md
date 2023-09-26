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

## Q2: Does `Cursor.try_next` return None if `getMore` is still in flight?
A: No? Blocking `getMore` with a failpoint results in this loop returning all documents:
```rust
while let Some(book) = cursor.try_next().await? {
    cnt += 1;
}
```

## Q1: Is `collection.find()` lazy?
A: No. `collection.find()` sends the "find" command on invocation. This differs from C driver. C driver does not send "find" command until the first iteration of the returned cursor.