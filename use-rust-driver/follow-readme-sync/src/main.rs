use mongodb::{
    bson::doc,
    options::{ClientOptions, ResolverConfig},
    sync::Client,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    title: String,
    author: String,
}

fn main() -> mongodb::error::Result<()> {
    let options = ClientOptions::parse_with_resolver_config(
        "mongodb://localhost:27017",
        ResolverConfig::cloudflare(),
    )?;
    let client = Client::with_options(options)?;

    // let client = Client::with_uri_str("mongodb://localhost:27017")?;
    let database = client.database("mydb");
    let collection = database.collection::<Book>("books");
    collection.drop(None)?;

    let docs = vec![
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

    // Insert some books into the "mydb.books" collection.
    collection.insert_many(docs, None)?;

    let cursor = collection.find(doc! { "author": "George Orwell" }, None)?;
    for result in cursor {
        println!("title: {}", result?.title);
    }
    return Ok(());
}
