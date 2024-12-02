To test, start a server with [test certificates](https://github.com/mongodb-labs/drivers-evergreen-tools/tree/93b20d9660fa5ef82b63d541d5a6f86f80ba4503/.evergreen/x509gen):

```bash
MONGODB=~/bin/mongodl/archive/8.0.0/mongodb-macos-aarch64-enterprise-8.0.0/bin/
CERTS=~/code/drivers-evergreen-tools/.evergreen/x509gen

$MONGODB/mongod \
    --tlsCAFile=$CERTS/ca.pem \
    --tlsCertificateKeyFile=$CERTS/server.pem \
    --tlsMode=requireTLS \
    --dbpath .menv
```

Update `dependencies.mongodb.path` in Cargo.toml to refer to a local checkout of the Rust driver with needed changes:

```toml
[dependencies.mongodb]
path = "/Users/kevin.albertson/review/mongo-rust-driver-1256"
features = ["openssl-tls"]
```

Update the local checkout of the Rust driver to include `3des`, `des-insecure`, and `sha1-insecure` flags in the `pkcs8` dependency to permit insecure algorithms for the test certificate:

```toml
pkcs8 = { version = "0.10.2", features = [
    "encryption",
    "pkcs5",
    "3des",
    "des-insecure",
    "sha1-insecure",
] }
```
