To start server:

```bash
# Before running, copy orchestration config to $DRIVERS_TOOLS/.evergreen/orchestration/configs/replica_sets

DRIVERS_TOOLS=$HOME/code/drivers-evergreen-tools

# Start servers:
cd $DRIVERS_TOOLS
TOPOLOGY=replica_set ORCHESTRATION_FILE=analytics-node-replset-config.json make run-server
```
