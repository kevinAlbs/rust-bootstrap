from pymongo import MongoClient
import os

certpath = os.environ["CERTPATH"]
uri = f"mongodb://localhost:27017/?tls=true&tlsCAFile={certpath}/ca.pem&tlsCertificateKeyFile={certpath}/client-pkcs8-encrypted.pem&tlsCertificateKeyFilePassword=password"
client = MongoClient(uri)
reply = client["db"].command({"ping": 1})
print (reply)
