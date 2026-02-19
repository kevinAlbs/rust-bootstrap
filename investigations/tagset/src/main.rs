use tokio;

use mongodb::{
    Client,
    bson::doc,
    options::{ClientOptions, ReadPreference, ReadPreferenceOptions, SelectionCriteria, TagSet},
};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let uri = "mongodb://localhost:27017/?replicaSet=repl0"; // Other nodes will be discovered by SDAM.
    let opts = ClientOptions::parse(uri).await?;

    let client = Client::with_options(opts)?;

    // Sleep to await initial discovery:
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Use selection criteria described in https://mongodb.slack.com/archives/CP893U9MX/p1771031511000619
    let selection_criteria =
        SelectionCriteria::ReadPreference(ReadPreference::SecondaryPreferred {
            options: Some(
                ReadPreferenceOptions::builder()
                    .tag_sets(Some(vec![TagSet::new()]))
                    .build(),
            ),
        });

    // Send replSetGetStatus. Expect to select secondary:
    let reply = client
        .database("admin")
        .run_command(doc! {"replSetGetStatus": 1})
        .selection_criteria(selection_criteria)
        .await?;
    let mut found_self = false;
    for member in reply.get_array("members").unwrap().clone() {
        let member_doc = member.as_document().unwrap();
        if member_doc.get_bool("self").unwrap_or(false) {
            found_self = true;
            let state_str = member_doc.get_str("stateStr").unwrap();
            if state_str != "SECONDARY" {
                println!(
                    "ERROR: expected to select SECONDARY, but selected {}",
                    state_str
                );
            } else {
                println!("OK: selected SECONDARY");
            }
        }
    }
    assert!(found_self);

    Ok(())
}
