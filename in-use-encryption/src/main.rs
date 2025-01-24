// To run:
//
// export MONGOCRYPT_LIB_DIR=/path/to/libmongocrypt.{so,dylib,dll}
// export CRYPT_SHARED_LIB_PATH=/path/to/crypt_shared.{so,dylib,dll}
// cargo run
//
use mongodb::{
    bson::{doc, Document},
    client_encryption::{ClientEncryption, LocalMasterKey},
    mongocrypt::ctx::KmsProvider,
    options::ClientOptions,
    Client, Namespace,
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let crypt_shared_path = std::env::var("CRYPT_SHARED_LIB_PATH").expect("Set CRYPT_SHARED_LIB_PATH environment variable to path to crypt_shared library");
    let uri = "mongodb://localhost:27017/";

    let key_vault_namespace = Namespace::new("keyvault", "datakeys");

    // Create KMS Providers:
    let kms_providers = vec![(
        KmsProvider::local(),
        doc! {
            "key": "mdwqyqBIhbqnvzH4pP2N9COUkXBvdrp3Yaw7Z/9rjbKO2ecXYwTNqIKHi71l70ZvDBX/IsvhhoJxDFExBY0j7Ysc1biWFAd3TcimiI2YW6TiAGpqKGcqvxrqXEjSLemG"
        },
        None,
    )];

    // Create a ClientEncryption:
    let ce: mongodb::client_encryption::ClientEncryption;
    {
        let key_vault_client = Client::with_uri_str(uri).await.unwrap();
        ce = ClientEncryption::new(
            key_vault_client,
            key_vault_namespace.clone(),
            kms_providers.clone(),
        )
        .unwrap();
    }

    // Create a data key:
    let key_id = ce
        .create_data_key(LocalMasterKey::builder().build())
        .await
        .unwrap();

    // Create schema:
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

    // Create a client with auto encryption enabled:
    let client;
    {
        let co = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .unwrap();
        let builder = Client::encrypted_builder(co, key_vault_namespace, kms_providers).expect("");
        client = builder
            .schema_map([("db.coll", schema)])
            .extra_options(Some(doc!{
                "cryptSharedLibPath": crypt_shared_path
            }))
            .build()
            .await
            .unwrap();
    }

    // Insert a document:
    {
        let coll = client.database("db").collection("coll");
        coll.drop().await.expect("should drop");
        coll.insert_one(doc! {"encryptedField": "foo"}).await?;
        println!("Inserted encrypted document into 'db.coll'");
    }

    // Find the document on an unencrypted client:
    {
        let unencrypted_coll = Client::with_uri_str("mongodb://localhost:27017")
            .await?
            .database("db")
            .collection::<Document>("coll");
        println!(
            "Encrypted document: {:?}",
            unencrypted_coll.find_one(doc! {}).await?
        );
    }

    return Ok(());
}
