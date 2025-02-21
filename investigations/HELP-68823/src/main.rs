use mongodb::{bson::doc, options::ClientOptions, Client};
use openssl;
use std::env;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    tracing_subscriber::fmt::init();
    println!("Using openssl version: {}", openssl::version::version());
    let uri = env::var("MONGODB_URI").expect("set MONGODB_URI");
    let opts = ClientOptions::parse(uri).await?;
    let client = Client::with_options(opts)?;
    let res = client
        .database("test")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("got: {:?}", res);
    client.shutdown().await;
    Ok(())
}
