// To run: Ensure `mongocryptd` is NOT on the PATH!
//
// $ export MONGOCRYPT_LIB_DIR=/path/to/libmongocrypt.{so,dylib,dll}
// $ cargo run
// 
// Gets error: "No such file or directory"
//
use mongodb::{
    bson::doc,
    client_encryption::{ClientEncryption, LocalMasterKey},
    mongocrypt::ctx::KmsProvider,
    options::ClientOptions,
    Client, Namespace,
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
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

    return Ok(());
}
