use mongodb::{
    bson::doc, bson::Document, client_encryption::ClientEncryption, client_encryption::MasterKey,
    error::Result, mongocrypt::ctx::KmsProvider, options::ClientOptions, Client, Namespace,
};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let key_vault_namespace = Namespace::new("keyvault", "datakeys");
    // Create KMS providers.
    let kms_providers;
    {
        let keydata = "mdwqyqBIhbqnvzH4pP2N9COUkXBvdrp3Yaw7Z/9rjbKO2ecXYwTNqIKHi71l70ZvDBX/IsvhhoJxDFExBY0j7Ysc1biWFAd3TcimiI2YW6TiAGpqKGcqvxrqXEjSLemG";
        kms_providers = vec![(
            KmsProvider::Local,
            doc! {
                "key": keydata
            },
            None,
        )];
    }

    let ce: mongodb::client_encryption::ClientEncryption;
    // Create a ClientEncryption.
    {
        let key_vault_client = Client::with_uri_str("mongodb://localhost:27017").await?;
        ce = ClientEncryption::new(
            key_vault_client,
            key_vault_namespace.clone(),
            kms_providers.clone(),
        )?;
    }

    // Create a data key.
    let key_id;
    {
        key_id = ce.create_data_key(MasterKey::Local).run().await?;
    }

    // Create schema.
    let schema = doc! {
        "properties": {
            "encryptedField": {
                "encrypt": {
                    "keyId": [key_id],
                    "bsonType": "string",
                    "algorithm": "AEAD_AES_256_CBC_HMAC_SHA_512-Deterministic",
                }
            }
        },
        "bsonType": "object",
    };

    let client;
    {
        let co = ClientOptions::parse("mongodb://localhost:27017").await?;
        client = Client::encrypted_builder(co, key_vault_namespace, kms_providers)?
            .schema_map([("db.coll", schema)])
            .build()
            .await?;
    }

    let coll = client.database("db").collection("coll");
    coll.insert_one(doc! {"encryptedField": "foo"}, None)
        .await?;
    println!("Inserted.");

    let unencrypted_coll = Client::with_uri_str("mongodb://localhost:27017")
        .await?
        .database("db")
        .collection::<Document>("coll");
    println!(
        "Encrypted document: {:?}",
        unencrypted_coll.find_one(None, None).await?
    );

    return Ok(());
}
