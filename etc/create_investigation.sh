if [[ -z "$NAME" ]]; then
	echo "Usage: NAME=<name>  create_investigation.sh";
	exit 1
fi

if [[ -d "investigations/$NAME" ]]; then
    echo "investigations/$NAME already exists"
    exit 1
fi

pushd investigations
cargo new $NAME
popd


# Append dependencies. Assumes last line in `Cargo.toml` is `[dependencies]`
cat <<EOF >> investigations/$NAME/Cargo.toml
serde = { version = "1.0", features = ["derive"] }
[dependencies.mongodb]
version = "3.2.3"
EOF

# TODO: Create async.
cat <<EOF > investigations/$NAME/src/main.rs

use mongodb::{
    bson::doc,
    options::ClientOptions,
    Client,
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let uri = "mongodb://localhost:27017";
    let opts = ClientOptions::parse(uri).await?;
    let client = Client::with_options(opts)?;
    let res = client
        .database("test")
        .run_command(doc!{"ping": 1})
        .await?;
    println!("got: {:?}", res);

    Ok(())
}

EOF
