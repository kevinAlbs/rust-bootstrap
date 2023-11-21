// Q: Does retrying a transaction after a transient error require first calling abort?
// A: Yes. Otherwise, the next transaction on the session may receive a "transaction in progress" error.

use mongodb::bson::doc;
use mongodb::Client;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let uri = "mongodb://localhost:27017";
    let client = Client::with_uri_str(uri).await?;

    let mut session = client.start_session(None).await?;
    let coll = client.database("db").collection("coll");

    // Run a transaction with a transient error.
    {
        // Set a failpoint to simlulate a transient error.
        // Failpoints are documented on: https://github.com/mongodb/mongo/wiki/The-%22failCommand%22-fail-point
        client
            .database("admin")
            .run_command(
                doc! {
                  "configureFailPoint": "failCommand",
                  "mode": {
                    "times": 1
                  },
                  "data": {
                    "failCommands": [
                      "insert"
                    ],
                    "closeConnection": true
                  }
                },
                None,
            )
            .await?;

        session
            .start_transaction(None)
            .await
            .expect("should start transaction");

        let res = coll
            .insert_one_with_session(doc!("x": 1), None, &mut session)
            .await;
        let err = res.err().expect("expected error, but got success");
        if !err.contains_label(mongodb::error::TRANSIENT_TRANSACTION_ERROR) {
            panic!(
                "Expected error to contain TRANSIENT_TRANSACTION_ERROR, got: {}",
                err
            );
        }
    }

    // Retry the transaction without aborting.
    {
        let res = session.start_transaction(None).await;

        let err = res.err().expect("expected error, but got success");
        if !err.to_string().contains("transaction already in progress") {
            panic!(
                "Expected error to contain 'transaction already in progress', got: {}",
                err
            );
        }
        // Abort to clean up.
        session.abort_transaction().await.expect("should abort");
    }

    // Successfully retry a transient transaction error:
    {
        // Set a failpoint to simlulate a transient error.
        // Failpoints are documented on: https://github.com/mongodb/mongo/wiki/The-%22failCommand%22-fail-point
        client
            .database("admin")
            .run_command(
                doc! {
                  "configureFailPoint": "failCommand",
                  "mode": {
                    "times": 1
                  },
                  "data": {
                    "failCommands": [
                      "insert"
                    ],
                    "closeConnection": true
                  }
                },
                None,
            )
            .await?;

        async fn run_transaction(
            mut session: &mut mongodb::ClientSession,
        ) -> mongodb::error::Result<()> {
            session
                .start_transaction(None)
                .await
                .expect("should start transaction");

            let coll = session.client().database("db").collection("coll");

            coll.insert_one_with_session(doc!("x": 1), None, &mut session)
                .await?;

            return session.commit_transaction().await;
        }

        let res = run_transaction(&mut session).await;
        if let Some(err) = res.err() {
            // If the `abort_transaction` is not called, a retry results in a "transaction in progress" error on `start_transaction`.
            session.abort_transaction().await?;
            if err.contains_label(mongodb::error::TRANSIENT_TRANSACTION_ERROR) {
                // Retry.
                run_transaction(&mut session).await?;
            } else {
                // Do not retry.
                return Err(err);
            }
        }
    }

    Ok(())
}
