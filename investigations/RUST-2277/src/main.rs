// Run with test.sh

use mongodb::{
    bson::doc,
    mongocrypt::ctx::KmsProvider,
    options::ClientOptions,
    Client, Namespace,
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let mongocryptd_path = std::env::var("MONGOCRYPTD_PATH").expect("Set MONGOCRYPTD_PATH environment variable to path to mongocryptd");

    let key_vault_namespace = Namespace::new("keyvault", "datakeys");

    // Create KMS Providers:
    let kms_providers = vec![(
        KmsProvider::local(),
        doc! {
            "key": "mdwqyqBIhbqnvzH4pP2N9COUkXBvdrp3Yaw7Z/9rjbKO2ecXYwTNqIKHi71l70ZvDBX/IsvhhoJxDFExBY0j7Ysc1biWFAd3TcimiI2YW6TiAGpqKGcqvxrqXEjSLemG"
        },
        None,
    )];

    // Create a client with auto encryption enabled:
    let client;
    {
        let co = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .unwrap();
        let builder = Client::encrypted_builder(co, key_vault_namespace, kms_providers).expect("");
        client = builder
            .extra_options(Some(doc!{
                "mongocryptdSpawnPath": mongocryptd_path
            }))
            .build()
            .await
            .unwrap();
    }

    // Insert a document:
    client.database("db").collection("coll").insert_one(doc! {"encryptedField": "foo"}).await?;
    println!("Test passed!");

    return Ok(());
}
