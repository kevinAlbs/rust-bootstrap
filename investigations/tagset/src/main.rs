use tokio;

use mongodb::{
    bson::doc,
    event::{command::CommandEvent, EventHandler, sdam},
    options::{ClientOptions,SelectionCriteria,TagSet,ReadPreference,ReadPreferenceOptions},
    Client,
};

use std::sync::{Arc, Mutex};

use std::collections::HashSet;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let uri = "mongodb://localhost:27017/?replicaSet=repl0"; // Other nodes will be discovered by SDAM.
    let mut opts = ClientOptions::parse(uri).await?;

    // Use a command started event to show which server was selected:
    opts.command_event_handler = Some(EventHandler::callback(move |ev| {
         match ev {
            CommandEvent::Started(ev) => {
                println!("Sending {} selected server {}", ev.command_name, ev.connection.address);
            }
            _ => (),
        }
    }));

    // Track server changed events:
    let counters = Arc::new(Mutex::new(HashSet::<String>::new()));
    let counters_ref = counters.clone();
    opts.sdam_event_handler = Some(EventHandler::callback(move |ev| {
        match ev {
            sdam::SdamEvent::ServerDescriptionChanged(ev) => {
                println!("Server Description changed for: {} from {} to {}", ev.new_description.address(), ev.previous_description.server_type(), ev.new_description.server_type());
                counters_ref.lock().unwrap().insert(ev.new_description.address().to_string());
            }
            _ => (),
        }
    }));
    
    let client = Client::with_options(opts)?;
    
    // Wait for all servers to initially be discovered:
    let reply = client.database("admin")
        .run_command(doc! { "replSetGetStatus": 1 })
        .await
        .unwrap();
    let node_count = reply.get_array("members").unwrap().len();

    let mut servers_discovered = counters.lock().unwrap().len();
    while servers_discovered < node_count {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        servers_discovered = counters.lock().unwrap().len();
    }

    // Use selection criteria described in https://mongodb.slack.com/archives/CP893U9MX/p1771031511000619
    let selection_criteria = {
        let analytics_tag_set: TagSet =
            TagSet::from([("nodeType".to_string(), "ANALYTICS".to_string())]);

        // An empty tag set is intended to select a secondary when there is no analytics node available.
        let empty_tag_set = TagSet::new();

        SelectionCriteria::ReadPreference(ReadPreference::SecondaryPreferred {
            options: Some(
                ReadPreferenceOptions::builder()
                    .tag_sets(Some(vec![empty_tag_set]))
                    .build(),
            ),
        })
    };

    let reply = client
        .database("admin")
        .run_command(doc!{"replSetGetStatus": 1}).selection_criteria(selection_criteria)
        .await?;

    println!("Expecting to select server with _id:0");
    let mut found_self = false;
    for member in reply.get_array("members").unwrap().clone() {
        let member_doc = member.as_document().unwrap();
        println!("{}", member_doc);
        if member_doc.get_bool("self").unwrap_or(false) {
            found_self = true;
            assert_eq!(member_doc.get_i32("_id").unwrap(), 0);
        }
    }
    assert!(found_self);

    Ok(())
}

