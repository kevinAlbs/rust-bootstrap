if [[ -z "$NAME" ]]; then
	echo "Usage: NAME=<name> [SYNC=ON] create_investigation.sh";
	exit 1
fi

if [[ -d "investigations/$NAME" ]]; then
    echo "investigations/$NAME already exists"
    exit 1
fi

pushd investigations
cargo new $NAME
popd

SYNC=${SYNC:-OFF}

if [[ "${SYNC}" == "ON" ]]; then
cat <<EOF > investigations/$NAME/src/main.rs
use mongodb::{bson::doc, sync::Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    title: String,
    author: String,
}

fn main() -> mongodb::error::Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017")?;
    let collection = client.database("db").collection::<Book>("books");
    collection.drop(None)?;

    let books = vec![
        Book {
            title: "1984".to_string(),
            author: "George Orwell".to_string(),
        },
        Book {
            title: "Animal Farm".to_string(),
            author: "George Orwell".to_string(),
        },
        Book {
            title: "The Great Gatsby".to_string(),
            author: "F. Scott Fitzgerald".to_string(),
        },
    ];

    // Insert some books into the "books" collection.
    collection.insert_many(books, None)?;

    let cursor = collection.find(doc! { "author": "George Orwell" }, None)?;
    for result in cursor {
        println!("title: {}", result?.title);
    }
    return Ok(());
}
EOF

# Append dependencies. Assumes last line in `Cargo.toml` is `[dependencies]`
cat <<EOF >> investigations/$NAME/Cargo.toml
serde = { version = "1.0", features = ["derive"] }
[dependencies.mongodb]
version = "2.6.0"
features = ["tokio-sync"]
EOF

exit 0
fi

# TODO: Create async.
cat <<EOF > investigations/$NAME/src/main.rs

use mongodb::bson::doc;
use mongodb::error::Result;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use tokio;

// This trait is required to use try_next() on the cursor
use futures::stream::TryStreamExt;

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    title: String,
    author: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;

    // Get a handle to a collection in the database.
    let collection = client.database("db").collection::<Book>("books");
    collection.drop(None).await?;

    let books = vec![
        Book {
            title: "1984".to_string(),
            author: "George Orwell".to_string(),
        },
        Book {
            title: "Animal Farm".to_string(),
            author: "George Orwell".to_string(),
        },
        Book {
            title: "The Great Gatsby".to_string(),
            author: "F. Scott Fitzgerald".to_string(),
        },
    ];

    // Insert the books into "books" collection.
    collection.insert_many(books, None).await?;

    let mut cursor = collection
        .find(doc! { "author": "George Orwell" }, None)
        .await?;
    while let Some(book) = cursor.try_next().await? {
        println!("title: {}", book.title);
    }

    return Ok(());
}

EOF

# Append dependencies. Assumes last line in `Cargo.toml` is `[dependencies]`
cat <<EOF >> investigations/$NAME/Cargo.toml
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
util = { path = "../../util" }
[dependencies.mongodb]
version = "2.6.0"
features = ["tokio-runtime"]
EOF
