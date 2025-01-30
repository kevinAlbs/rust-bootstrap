use mongodb::{bson::doc, options::ClientOptions, Client};
use std::env;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let ca_path = env::var("CA_PATH").expect("set CA_PATH to path of drivers-evergreen-tools/.evergreen/x509gen/ca.pem");
    let client_path = env::var("CLIENT_PATH").expect("set CLIENT_PATH to path of client certificate");
    let client_password = env::var("CLIENT_PASSWORD").expect("set CLIENT_PASSWORD to client certificate password");
    let uri = format!("mongodb://localhost:27017/?tls=true&tlsCAFile={ca_path}&tlsCertificateKeyFile={client_path}&tlsCertificateKeyFilePassword={client_password}&serverSelectionTimeoutMS=1000");
    let opts = ClientOptions::parse(uri).await?;
    let client = Client::with_options(opts)?;
    let res = client
        .database("test")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("got: {:?}", res);
    Ok(())
}
