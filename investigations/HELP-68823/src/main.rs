use mongodb::{bson::doc, options::ClientOptions, Client};
use std::env;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let certpath = env::var("CERTPATH").expect("set CERTPATH to checkout of drivers-evergreen-tools/.evergreen/x509gen");
    let uri = format!("mongodb://[::1]:27017/?tls=true&tlsCAFile={certpath}/ca.pem&tlsCertificateKeyFile={certpath}/client-pkcs8-encrypted.pem&tlsCertificateKeyFilePassword=password&serverSelectionTimeoutMS=2000");
    let opts = ClientOptions::parse(uri).await?;
    let client = Client::with_options(opts)?;
    let res = client
        .database("test")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("got: {:?}", res);
    Ok(())
}
