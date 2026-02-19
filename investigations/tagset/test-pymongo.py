from pymongo import MongoClient, read_preferences  
from time import sleep

# Create a MongoClient instance  
client = MongoClient("mongodb://localhost:27017/?replicaSet=repl0")
  
# Set up secondaryPreferred read preference with an empty tag set
read_pref = read_preferences.SecondaryPreferred(tag_sets=[{}])

# Sleep to await initial discovery.
sleep(1)

# Send replSetGetStatus. Expect to select secondary:
reply = client["admin"].command({"replSetGetStatus": 1}, read_preference=read_pref)
found_self = False
for member in reply["members"]:
    if "self" in member and member["self"]:
        found_self = True
        if member["stateStr"] != "SECONDARY":
            print("ERROR: expected to select SECONDARY, but selected " + member["stateStr"])
        else:
            print("OK: selected SECONDARY")
if not found_self:
    print("ERROR: could not find self")
