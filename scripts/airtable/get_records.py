import json
import requests
import sys
import os

arguments = json.loads(sys.argv[1])
api_key = os.environ.get('AIRTABLE_API_KEY')
url = f'https://api.airtable.com/v0/{arguments["baseid"]}/{arguments["tableid"]}'
headers = {
    'Authorization': f'Bearer {api_key}',
    'Content-Type': 'application/json',
}
response = requests.get(url, headers=headers)
print(response.text)
