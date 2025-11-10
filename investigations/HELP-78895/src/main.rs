use tokio;

use mongodb::{
    bson::doc,
    options::{ClientOptions, ResolverConfig},
    Client,
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI environment variable must be set");
    println!("Testing with URI: {}", uri);

    {
        print!("Testing default resolver ... ");
        let opts = ClientOptions::parse(uri.clone()).await?;
        let client = Client::with_options(opts)?;
        let res = client
            .database("test")
            .run_command(doc!{"ping": 1})
            .await;
        if res.is_err() {
            println!("failed: {:?}", res.err());
        } else {
            println!("succeeded!");
        }
    }

    {
        print!("Testing CloudFlare resolver ... ");
        let opts = ClientOptions::parse(uri.clone()).resolver_config(ResolverConfig::cloudflare()).await?;
        let client = Client::with_options(opts)?;
        let res = client
            .database("test")
            .run_command(doc!{"ping": 1})
            .await;
        if res.is_err() {
            println!("failed: {:?}", res.err());
        } else {
            println!("succeeded!");
        }
    }

    Ok(())
}

