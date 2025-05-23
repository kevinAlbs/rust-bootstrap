use futures::StreamExt;
use mongodb::{
    bson::{Document, doc},
    options::ClientOptions,
    Client, Collection
};


#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    tracing_subscriber::fmt::init();    
    let uri = "mongodb://localhost:27017/";
    let opts = ClientOptions::parse(uri).await?;
    let client = Client::with_options(opts)?;
    
    let coll : Collection<Document> = client.database("db").collection("coll");
    let docs = vec![doc!{"foo": "bar"}, doc!{"foo": "baz"}];
    coll.insert_many(docs).await.expect("should insert");

    let mut cs = coll.watch().batch_size(0).await.expect("should establish change stream");
    let res = cs.next().await.expect("should iterate").expect("should iterate");
    println!("got: {}", res.full_document.unwrap());


    Ok(())
}
