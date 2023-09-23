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

# Q&A

## Q2: Does `Cursor.try_next` return None if `getMore` is still in flight?
A: No? Blocking `getMore` with a failpoint results in this loop returning all documents:
```rust
while let Some(book) = cursor.try_next().await? {
    cnt += 1;
}
```

## Q1: Is `collection.find()` lazy?
A: No. `collection.find()` sends the "find" command on invocation. This differs from C driver. C driver does not send "find" command until the first iteration of the returned cursor.