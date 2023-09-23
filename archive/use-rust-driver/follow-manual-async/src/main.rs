use futures::TryStreamExt;
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
    if val == 666 {
        if cfg!(panic = "abort") {
            println!("This is not your party. Run!!!!");
        } else {
            println!("Spit it out!!!!");
        }
    }

    let res = ce
        .encrypt(val, keyid.clone(), Algorithm::Unindexed)
        .run()
        .await
        .expect("should succeed");
    return res;
}

async fn foo(x: Option<i32>) {
    let res = x.map(|v| v + 1);
    if res.is_some() {
        println!("res is {}", res.unwrap());
    }
    let got = returns_option().await;
    println!("{:?}", got.map(|v| { v + 1 }));
}

async fn returns_option() -> Option<i32> {
    return Some(123);
}

#[tokio::main]
async fn main() -> Result<()> {
    const URI: &str = "mongodb://localhost:27017";

    foo(Some(123)).await;
    foo(None).await;

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
