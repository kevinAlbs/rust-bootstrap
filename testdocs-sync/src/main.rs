use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
};

use mongodb;

fn main() -> mongodb::error::Result<()> {
    let uri = "mongodb://[::1]:27017/";
    let mut client_options = ClientOptions::parse(uri)?;

    // start-stable-api
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    let client: mongodb::sync::Client = mongodb::sync::Client::with_options(client_options)?;
    // end-stable-api
    
    client
        .database("admin")
        .run_command(doc! { "ping": 1 }, None)?;
    println!("Pinged your deployment. You successfully connected to MongoDB!");

    Ok(())
}
