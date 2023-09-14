use mongodb::{
    bson::doc, error::Result, mongocrypt::ctx::KmsProvider, options::ClientOptions, Client,
    Namespace,
};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let keydata = "mdwqyqBIhbqnvzH4pP2N9COUkXBvdrp3Yaw7Z/9rjbKO2ecXYwTNqIKHi71l70ZvDBX/IsvhhoJxDFExBY0j7Ysc1biWFAd3TcimiI2YW6TiAGpqKGcqvxrqXEjSLemG";
    let co = ClientOptions::builder().build();
    let kms_providers = [(KmsProvider::Local, doc! { "key": keydata }, None)];
    let key_vault_namespace = Namespace::new("keyvault", "datakeys");
    let ec = Client::encrypted_builder(co, key_vault_namespace, kms_providers)?
        .extra_options(doc! {
            "bypassAutoEncryption": true
        })
        .build()
        .await?;
    Ok(())
}
