use tokio;

use mongodb::{
    bson::doc,
    options::ClientOptions,
    Client,
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI environment variable must be set");
    let opts = ClientOptions::parse(uri).await?;
    let client = Client::with_options(opts)?;
    let res = client
        .database("test")
        .run_command(doc!{"ping": 1})
        .await?;
    println!("got: {:?}", res);

    Ok(())
}

