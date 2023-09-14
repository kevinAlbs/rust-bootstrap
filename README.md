# Q2: Does `Cursor.try_next` return None if `getMore` is still in flight?
A: No? Blocking `getMore` with a failpoint results in this loop returning all documents:
```rust
while let Some(book) = cursor.try_next().await? {
    cnt += 1;
}
```

# Q1: Is `collection.find()` lazy?
A: No. `collection.find()` sends the "find" command on invocation. This differs from C driver. C driver does not send "find" command until the first iteration of the returned cursor.