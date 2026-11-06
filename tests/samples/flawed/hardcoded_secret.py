import requests

API_KEY = "example-not-a-real-key-0000"

def fetch_report():
    return requests.get("https://api.example.com/report", headers={"Authorization": "Bearer " + API_KEY})
