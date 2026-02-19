from pymongo import MongoClient, read_preferences  
from pymongo.monitoring import CommandListener, CommandStartedEvent
from time import sleep
  
# Connection URI to your MongoDB cluster  
uri = "mongodb://localhost:27017/?replicaSet=repl0"
  
# Define a custom CommandListener to monitor commands  
class MyCommandListener(CommandListener):  
    def started(self, event: CommandStartedEvent):  
        # Print information about the server selected for each command  
        print(f"Selected server: {event.connection_id}")  
  
    def succeeded(self, event):  
        pass
  
    def failed(self, event):  
        pass
  
# Create a MongoClient instance  
client = MongoClient(uri, event_listeners=[MyCommandListener()])  
  
# Set up secondaryPreferred read preference with tag set  
read_pref = read_preferences.SecondaryPreferred(tag_sets=[{"nodeType": "analytics"}, {}])

# Sleep to await initial discovery.
sleep(1)
  
# Ping command using the preferred server
try:  
    client["admin"].command({"replSetGetStatus": 1}, read_preference=read_pref)
    print("Ping succeeded")
except Exception as e:  
    print("Error:", e)  
