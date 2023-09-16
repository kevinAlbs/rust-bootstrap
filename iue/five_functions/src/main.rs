// An example running the five main functions of In-Use Encryption (IUE).
// Run with: cargo run
// Include path to `mongocryptd` in `PATH`.
// Include path to `libmongocrypt` in `MONGOCRYPT_LIB_DIR`.
//
// Set the environment variable KMS_PROVIDERS_PATH to the path of a JSON file with KMS credentials.
// KMS_PROVIDERS_PATH defaults to ~/.iue/kms_providers.json.
//
// Set the environment variable MONGODB_URI to set a custom URI. MONGODB_URI defaults to
// mongodb://localhost:27017.

use mongodb::{
    bson::{self, doc, Binary, Document},
    client_encryption::{ClientEncryption, MasterKey},
    error::Result,
    mongocrypt::ctx::Algorithm,
    mongocrypt::ctx::KmsProvider,
    Client, Namespace,
};
use rand::Rng;

async fn encrypt_value(ce: &ClientEncryption, keyid: &Binary, val: i32) -> Binary {
    let res = ce
        .encrypt(val, keyid.clone(), Algorithm::Unindexed)
        .run()
        .await
        .expect("should succeed");
    return res;
}

use std::path::Path;
use std::path::PathBuf;

async fn read_kms_providers(
    path: &Path,
) -> std::result::Result<Document, Box<dyn std::error::Error>> {
    // Read file.
    let contents = std::fs::read_to_string(path)?;
    // TODO: can I wrap the error message with more helpful context?
    // Parse contents into a serde_json::Map.
    let parsed: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&contents)?;
    // Parse serde_json::Map to a bson::Document.
    let doc = bson::Document::try_from(parsed)?;
    return Ok(doc);
}

#[tokio::main]
async fn main() -> Result<()> {
    const URI: &str = "mongodb://localhost:27017";

    // Read KMS providers from JSON file.
    {
        let path: PathBuf = {
            let path_from_env = std::env::var("KMS_PROVIDERS_PATH");
            match path_from_env {
                Ok(path_str) => PathBuf::from(path_str),
                Err(err) => {
                    match err {
                        std::env::VarError::NotPresent => {
                            // Try to apply default path: <home>/.iue/kms_providers.json.
                            match dirs::home_dir() {
                                Some(path_buf) => path_buf.join(".iue").join("kms_providers.json"),
                                None => {
                                    panic!("Unable to determine path to KMS providers file. KMS_PROVIDERS_PATH not set. Unable to apply default path <home>/.iue/kms_providers.json");
                                }
                            }
                        }
                        _ => {
                            panic!("Unable to read environment variable: KMS_PROVIDERS_PATH. Error: {}", err);
                        }
                    }
                }
            }
        };

        let res = read_kms_providers(&path).await;
        if !res.is_ok() {
            println!(
                "Failed to read KMS providers document at path {}. Error: {}",
                path.to_str().unwrap_or("<unable to read>"),
                res.err().unwrap()
            );
        }
    }

    let mut key_bytes = vec![0u8; 96];
    rand::thread_rng().fill(&mut key_bytes[..]);
    let local_master_key = bson::Binary {
        subtype: bson::spec::BinarySubtype::Generic,
        bytes: key_bytes,
    };
    let kms_providers = vec![(KmsProvider::Local, doc! { "key": local_master_key }, None)];
    let key_vault_namespace = Namespace::new("keyvault", "datakeys");
    let key_vault_client = Client::with_uri_str(URI).await?;
    let key_vault = key_vault_client
        .database(&key_vault_namespace.db)
        .collection::<Document>(&key_vault_namespace.coll);
    key_vault.drop(None).await?;
    let client_encryption = ClientEncryption::new(
        key_vault_client,
        key_vault_namespace.clone(),
        kms_providers.clone(),
    )?;
    let key1_id = client_encryption
        .create_data_key(MasterKey::Local)
        .key_alt_names(["firstName".to_string()])
        .run()
        .await?;

    // Attempt to encrypt a QE value with keyAltName.
    let got = encrypt_value(&client_encryption, &key1_id, 666).await;
    println!("encrypted to value: {}", got);
    return Ok(());
}
