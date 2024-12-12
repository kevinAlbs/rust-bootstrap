To test, start a server with [test certificates](https://github.com/mongodb-labs/drivers-evergreen-tools/tree/93b20d9660fa5ef82b63d541d5a6f86f80ba4503/.evergreen/x509gen):

```bash
MONGODB="$HOME/bin/mongodl/archive/8.0.0/mongodb-macos-aarch64-enterprise-8.0.0/bin/"
CERTPATH="$HOME/code/drivers-evergreen-tools/.evergreen/x509gen"

$MONGODB/mongod \
    --tlsCAFile=$CERTPATH/ca.pem \
    --tlsCertificateKeyFile=$CERTPATH/server.pem \
    --tlsMode=requireTLS \
    --dbpath .menv
```

Extract the username from the certificate:
```bash
openssl x509 -in $CERTPATH/client.pem -inform PEM -subject -nameopt RFC2253 -noout
# Prints:
# subject=C=US,ST=New York,L=New York City,O=MDB,OU=Drivers,CN=client
```

Connect with `mongosh`:
```
mongosh --tls --tlsCertificateKeyFile $CERTPATH/client.pem \
    --tlsCAFile $CERTPATH/ca.pem
```

Create a user with X509 auth:
```javascript
db.getSiblingDB("$external").runCommand(
  {
    createUser: "C=US,ST=New York,L=New York City,O=MDB,OU=Drivers,CN=client",
    roles: [
         { role: "readWrite", db: "test" },
         { role: "userAdminAnyDatabase", db: "admin" }
    ]
  }
)
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
export $CERTPATH
cargo run
```

Expect a successful ping:
```
got: Document({"ok": Double(1.0)})
```
