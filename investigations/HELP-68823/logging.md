In Cargo.toml, add `tracing-unstable` to mongodb features, and add dependency for `tracing-subscriber`:
```toml
# Cargo.toml
[dependencies.mongodb]
git = "https://github.com/mongodb/mongo-rust-driver.git"
rev = "6ad6c6ec" # Commit on ipv6-backport
features = ["openssl-tls", "cert-key-password", "tracing-unstable"]
# ...
[dependencies]
tracing-subscriber = "0.3"
```

Update app to add tracing:
```rust
fn main() {
    tracing_subscriber::fmt::init();
    // ...
}
```

Run with the environment variable to print trace:
```bash
RUST_LOG='mongodb=trace' cargo run
```

This should print logs like:
```bash
2025-02-21T15:27:00.725996Z DEBUG mongodb::connection: Connection checkout started topologyId="67b89b4429449d6239530fef" serverHost="localhost" serverPort=27017
2025-02-21T15:27:00.726081Z DEBUG mongodb::connection: Connection created topologyId="67b89b4429449d6239530fef" serverHost="localhost" serverPort=27017 driverConnectionId=1
2025-02-21T15:27:00.729758Z DEBUG mongodb::connection: Connection ready topologyId="67b89b4429449d6239530fef" serverHost="localhost" serverPort=27017 driverConnectionId=1 durationMS=3
2025-02-21T15:27:00.729877Z DEBUG mongodb::connection: Connection checked out topologyId="67b89b4429449d6239530fef" serverHost="localhost" serverPort=27017 driverConnectionId=1 durationMS=3
2025-02-21T15:27:00.730127Z DEBUG mongodb::command: Command started topologyId="67b89b4429449d6239530fef" command="{\"ping\":1,\"$db\":\"test\",\"lsid\":{\"id\":{\"$binary\":{\"base64\":\"T1uk5r+UTS2U9nxrUma6pw==\",\"subType\":\"04\"}}},\"$clusterTime\":{\"clusterTime\":{\"$timestamp\":{\"t\":1740151611,\"i\":1}},\"signature\":{\"hash\":{\"$binary\":{\"base64\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAA=\",\"subType\":\"00\"}},\"keyId\":0}}}" databaseName="test" commandName="ping" requestId=4 driverConnectionId=1 serverConnectionId=87 serverHost="localhost" serverPort=27017
2025-02-21T15:27:00.731260Z DEBUG mongodb::command: Command succeeded topologyId="67b89b4429449d6239530fef" reply="{\"ok\":1.0,\"$clusterTime\":{\"clusterTime\":{\"$timestamp\":{\"t\":1740151611,\"i\":1}},\"signature\":{\"hash\":{\"$binary\":{\"base64\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAA=\",\"subType\":\"00\"}},\"keyId\":0}},\"operationTime\":{\"$timestamp\":{\"t\":1740151611,\"i\":1}}}" commandName="ping" requestId=4 driverConnectionId=1 serverConnectionId=87 serverHost="localhost" serverPort=27017 durationMS=1
2025-02-21T15:27:00.731352Z DEBUG mongodb::connection: Connection checked in topologyId="67b89b4429449d6239530fef" serverHost="localhost" serverPort=27017 driverConnectionId=1
2025-02-21T15:27:00.731471Z DEBUG mongodb::connection: Connection closed topologyId="67b89b4429449d6239530fef" serverHost="localhost" serverPort=27017 driverConnectionId=1 reason="Connection pool was closed"
2025-02-21T15:27:00.731529Z DEBUG mongodb::connection: Connection pool closed topologyId="67b89b4429449d6239530fef" serverHost="localhost" serverPort=27017
```

Run once with a failure (with `--release`?) and once with a success. That might help identify the cause of connection close. Support can post logs in the HELP ticket.


Recommend: on retry, recreate client.
