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
    // Print each server configuration by direct connecting:
    for host in ["localhost:27017", "localhost:27018", "localhost:27019"] {
        let client = Client::with_options(ClientOptions::parse(format!("mongodb://{}/?directConnection=true", host)).await.unwrap()).unwrap();
        let reply = client.database("admin").run_command(doc!{"hello": 1}).await.unwrap();
        println!("{}: isWritablePrimary:{}, tags={:?}", host, reply["isWritablePrimary"], reply["tags"]);    
    }


    let uri = "mongodb://localhost:27017,localhost:27018,localhost:27019/?replicaSet=repl0";
    let mut opts = ClientOptions::parse(uri).await?;
    // Use a command started event to show which server was selected:
    opts.command_event_handler = Some(EventHandler::callback(move |ev| {
         match ev {
            CommandEvent::Started(ev) => {
                println!("selected server {}", ev.connection.address);
            }
            _ => (),
        }
    }));

    let counters = Arc::new(Mutex::new(HashSet::<String>::new()));

    let counters_ref = counters.clone();
    // Wait for all servers to initially be discovered
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

    // Use selection criteria described in https://mongodb.slack.com/archives/CP893U9MX/p1771031511000619
    let selection_criteria = {
        let analytics_tag_set: TagSet =
            TagSet::from([("nodeType".to_string(), "ANALYTICS".to_string())]);

        // This ensures that a secondary is chosen even if there isn't an analytics node available.
        let empty_tag_set = TagSet::new();

        SelectionCriteria::ReadPreference(ReadPreference::SecondaryPreferred {
            options: Some(
                ReadPreferenceOptions::builder()
                    .tag_sets(Some(vec![analytics_tag_set, empty_tag_set]))
                    .build(),
            ),
        })
    };

    // Await initial discovery of all three servers:
    {
        let mut servers_discovered = counters.lock().unwrap().len();
        while servers_discovered < 3 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            servers_discovered = counters.lock().unwrap().len();
        }
    }

    client
        .database("admin")
        .run_command(doc!{"replSetGetStatus": 1}).selection_criteria(selection_criteria)
        .await?;
    

    Ok(())
}

