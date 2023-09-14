use mongodb::error::Result;
use mongodb::{options::ClientOptions, Client};
use std::sync::Arc;
use tokio;

struct MyError {}

// Convert custom error to MongoDB Error:
// impl From<MyError> for mongodb::error::Error {
//     fn from(me: MyError) -> Self {
//         return mongodb::error::Error::custom(());
//     }
// }

impl From<MyError> for mongodb::error::ErrorKind {
    fn from(_me: MyError) -> Self {
        return mongodb::error::ErrorKind::Custom(Arc::new(()));
    }
}

async fn list_database_names() -> Result<()> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // List the names of the databases in that deployment.
    for db_name in client.list_database_names(None, None).await? {
        if db_name == "bar" {
            return Err(MyError {}.into());
        }
        println!("{}", db_name);
    }
    return Result::Ok(());
}

#[tokio::main]
// Q: What Result type do I need to return?
async fn main() -> Result<()> {
    let res = list_database_names().await;
    if res.is_err() {
        println!("got error: {:?}", res.unwrap_err());
    }
    return Result::Ok(());
}
