To start server:

```bash
# Before running, clone github.com/mongodb-labs/drivers-evergreen-tools to $DRIVERS_TOOLS
DRIVERS_TOOLS=$HOME/code/drivers-evergreen-tools
cp primary-and-secondary-config.json $DRIVERS_TOOLS/.evergreen/orchestration/configs/replica_sets
cp analytics-node-replica-set-config.json.json $DRIVERS_TOOLS/.evergreen/orchestration/configs/replica_sets

# Start servers:
cd $DRIVERS_TOOLS
TOPOLOGY=replica_set ORCHESTRATION_FILE=analytics-node-replset-config.json make run-server
```


