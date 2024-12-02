use mongodb::{
    bson::doc,
    options::{ClientOptions},
    Client,
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let uri = "mongodb+srv://test1.kevinalbs.com";
    let client_options = ClientOptions::parse(uri).await?;
    let client : mongodb::Client = Client::with_options(client_options)?;
    
    client.database("foo").run_command(doc!{"ping": 1}, None).await?;
    Ok(())
}
