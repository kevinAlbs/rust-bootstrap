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
    bson::{self, doc, Document},
    client_encryption::{ClientEncryption, MasterKey},
    mongocrypt::ctx::{Algorithm, KmsProvider},
    options::{ClientOptions, TlsOptions},
    Client, Namespace,
};

use std::path::PathBuf;

async fn get_kms_providers() -> Vec<(KmsProvider, Document, Option<TlsOptions>)> {
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
                        panic!(
                            "Unable to read environment variable: KMS_PROVIDERS_PATH. Error: {}",
                            err
                        );
                    }
                }
            }
        }
    };

    // Read file.
    let contents = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "should read file: {}",
            path.to_str().unwrap_or("<cannot read path>")
        );
    });

    // Parse contents into a serde_json::Map.
    let parsed: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&contents).expect("should parse as JSON");

    // Parse serde_json::Map to a bson::Document.
    let doc = bson::Document::try_from(parsed).expect("should convert JSON to BSON");

    // Construct a Vec<(KmsProvider, Document)> from the Document.
    let mut kms_providers: Vec<(KmsProvider, Document, Option<TlsOptions>)> = Vec::new();
    for (k, v) in doc.iter() {
        let kms_provider = KmsProvider::from_name(k);
        if let KmsProvider::Other(s) = kms_provider {
            panic!("Unexpected KMS provider: {}.", s)
        }
        let kms_provider_doc = v.as_document().unwrap_or_else(|| {
            panic!(
                "expected document for {}, got: {:?}",
                k.as_str(),
                v.element_type()
            );
        });
        kms_providers.push((kms_provider, kms_provider_doc.clone(), None));
    }
    kms_providers
}

#[tokio::main]
async fn main() {
    let uri = std::env::var("MONGODB_URI").unwrap_or("mongodb://localhost:27017".to_string());

    // Read KMS providers from JSON file.
    let kms_providers = get_kms_providers().await;
    let key_vault_namespace = Namespace::new("keyvault", "datakeys");
    let key_vault_client = Client::with_uri_str(&uri)
        .await
        .expect("should create Client");

    // Create a ClientEncryption struct.
    // A ClientEncryption struct provides admin helpers with three functions:
    // 1. create a data key
    // 2. explicit encrypt
    // 3. explicit decrypt
    let ce = ClientEncryption::new(
        key_vault_client,
        key_vault_namespace.clone(),
        kms_providers.clone(),
    )
    .expect("should create ClientEncryption");

    println!("CreateDataKey... begin");
    let keyid = ce
        .create_data_key(MasterKey::Local)
        .run()
        .await
        .expect("should create data key");
    println!("Created key with a UUID: {}", keyid);
    println!("CreateDataKey... end");

    println!("Encrypt... begin");
    let ciphertext = ce
        .encrypt(
            "test",
            keyid.clone(),
            Algorithm::AeadAes256CbcHmacSha512Deterministic,
        )
        .run()
        .await
        .expect("should encrypt");
    println!("Explicitly encrypted to ciphertext: {:?}", ciphertext);
    println!("Encrypt... end");

    println!("Decrypt... begin");
    let plaintext = ce
        .decrypt(ciphertext.as_raw_binary())
        .await
        .expect("should decrypt");
    println!("Explicitly decrypted to plaintext: {:?}", plaintext);
    println!("Decrypt... end");

    let schema = doc! {
        "properties": {
            "encryptMe": {
                "encrypt": {
                    "keyId": [keyid],
                    "bsonType": "string",
                    "algorithm": "AEAD_AES_256_CBC_HMAC_SHA_512-Deterministic",
                }
            }
        },
        "bsonType": "object",
    };

    let client = Client::encrypted_builder(
        ClientOptions::parse(&uri).await.expect("should parse URI"),
        key_vault_namespace,
        kms_providers,
    )
    .expect("should create builder")
    .schema_map([("db.coll".to_string(), schema)])
    .build()
    .await
    .expect("should build encrypted client");

    let coll = client.database("db").collection("coll");
    coll.drop(None).await.expect("should drop");

    println!("Automatic encryption ... begin");
    coll.insert_one(doc! {"encryptMe": "test"}, None)
        .await
        .expect("should insert");
    println!("Automatic encryption ... end");

    println!("Automatic decryption ... begin");
    let res = coll
        .find_one(doc! {}, None)
        .await
        .expect("should find result")
        .expect("should find document");
    println!("Decrypted document: {:?}", res);
    println!("Automatic decryption ... end");
}
