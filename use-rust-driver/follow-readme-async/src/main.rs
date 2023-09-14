// Following the async tutorial required:
// Determining the return type of `main`.

use mongodb::bson::{doc, Document};
use mongodb::event::command::CommandEventHandler;
use mongodb::event::command::{CommandFailedEvent, CommandStartedEvent, CommandSucceededEvent};
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use std::sync::Arc;
use tokio;

// This trait is required to use `try_next()` on the cursor
use futures::stream::TryStreamExt;

use mongodb::error::Result;

struct CommandPrinter {}

impl CommandEventHandler for CommandPrinter {
    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command is initiated.
    fn handle_command_started_event(&self, event: CommandStartedEvent) {
        println!("command started => {:?}", event.command);
    }

    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command successfully completes.
    fn handle_command_succeeded_event(&self, _event: CommandSucceededEvent) {}

    /// A [`Client`](../../struct.Client.html) will call this method on each registered handler
    /// whenever a database command fails to complete successfully.
    fn handle_command_failed_event(&self, _event: CommandFailedEvent) {}
}

#[tokio::main]
// Q: What Result type do I need to return?
async fn main() -> Result<()> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;

    client_options.command_event_handler = Some(Arc::new(CommandPrinter {}));

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // List the names of the databases in that deployment.
    for db_name in client.list_database_names(None, None).await? {
        println!("{}", db_name);
    }

    // Get a handle to a database.
    let db = client.database("keyvault");

    // List the names of the collections in that database.
    for collection_name in db.list_collection_names(None).await? {
        println!("-{}", collection_name);
    }

    // Get a handle to a collection in the database.
    let collection = db.collection::<Document>("books");
    collection.drop(None).await?;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Book {
        title: String,
        author: String,
    }

    // Get a handle to a collection of `Book`.
    let typed_collection = db.collection::<Book>("books");

    let mut books = vec![];

    // Insert more documents than can be handled in a batch.
    for i in 0..101 {
        books.push(Book {
            title: format!("Book {}", i),
            author: "George Orwell".to_string(),
        });
    }

    // Insert the books into "mydb.books" collection, no manual conversion to BSON necessary.
    typed_collection.insert_many(books, None).await?;

    // Query the books in the collection with a filter and an option.
    let filter = doc! { "author": "George Orwell" };
    let find_options = FindOptions::builder()
        .sort(doc! { "title": 1 })
        .batch_size(10)
        .build();

    // Q: does `find` immediately send a `find` command? Or is it lazy?
    // A: Yes. It is not lazy.
    let mut cursor = typed_collection.find(filter, find_options).await?;

    // Set failPoint on getMore to block 1000ms.
    client
        .database("admin")
        .run_command(
            doc! {
                "configureFailPoint": "failCommand",
                "mode": {"times": 1},
                "data": {
                    "failCommands": ["getMore"],
                    "blockConnection": true,
                    "blockTimeMS": 1000
                }
            },
            None,
        )
        .await?;

    let mut cnt = 0;
    // Iterate over the results of the cursor.
    // Q: does `Cursor.try_next` return None if `getMore` is still in flight?
    // A: No? Blocking `getMore` with a failpoint results in this loop returning all documents:
    while let Some(book) = cursor.try_next().await? {
        cnt += 1;
    }

    println!("got {} books", cnt);

    return Result::Ok(());
}

#[test]
fn testit() {
    let x = 123;
    assert_eq!(1, x);
}
