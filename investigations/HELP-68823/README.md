To test, start a server with [test certificates](https://github.com/mongodb-labs/drivers-evergreen-tools/tree/93b20d9660fa5ef82b63d541d5a6f86f80ba4503/.evergreen/x509gen):

```bash
export MONGODB="$HOME/mongodb-linux-x86_64-enterprise-ubuntu2004-4.4.29/bin/"
export CERTPATH="$HOME/drivers-evergreen-tools/.evergreen/x509gen"

$MONGODB/mongod \
    --tlsCAFile=$CERTPATH/ca.pem \
    --tlsCertificateKeyFile=$CERTPATH/server.pem \
    --tlsMode=requireTLS \
    --dbpath .menv \
    --ipv6
```

Update `dependencies.mongodb.path` in Cargo.toml to refer to a commit of the Rust driver with needed changes. Enable the `cert-key-password` feature:

```toml
[dependencies.mongodb]
git = "https://github.com/mongodb/mongo-rust-driver.git"
# Commit on ipv6-backport:
rev = "732dc54b"
features = ["openssl-tls", "cert-key-password"]
```

The test certificates are encrypted with the insecure 3DES algorithm. To permit insecure algorithms for the test certificate, add the `pkcs8` dependency with needed feature flags:

```toml
# Add pkcs8 with feature flags to enable insecure algorithms.
# Due to "Feature unification", this enables feature flags for mongodb driver.
[dependencies.pkcs8]
version = "0.10.2"
features = ["3des", "des-insecure", "sha1-insecure"]
```

Run:
```bash
export CERTPATH
cargo run
```

Expect a successful ping:
```
got: Document({"ok": Double(1.0)})
```
