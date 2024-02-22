use mongodb::bson::doc;
use mongodb::error::Result;
use mongodb::event::command::CommandEvent;
use mongodb::event::EventHandler;
use mongodb::options::ClientOptions;
use mongodb::Client;
use tokio;

// This trait is required to use try_next() on the cursor
use futures::stream::TryStreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse a connection string into an options struct.
    let client_options = ClientOptions::builder()
        .command_event_handler(EventHandler::callback(|ev| match ev {
            CommandEvent::Started(se) => {
                println!("command started: {:?}", se.command);
            }
            _ => {}
        }))
        .build();

    let client = Client::with_options(client_options).expect("should construct");

    let db = client.database("db");

    // Create a collection by inserting.
    db.collection("coll1")
        .insert_one(doc! {"foo": "bar"}, None)
        .await
        .expect("should insert");

    let colls: Result<Vec<_>> = db
        .list_collections()
        .authorized_collections(true)
        .await
        .unwrap()
        .try_collect()
        .await;
    assert_eq!(colls.unwrap()[0].name, "coll1");

    let colls: Result<Vec<_>> = db
        .list_collection_names()
        .authorized_collections(true)
        .await;
    assert_eq!(colls.unwrap()[0], "coll1");

    return Ok(());
}
