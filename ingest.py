import requests

# 1. On lit le fichier Markdown local
with open("wireguard.md", "r", encoding="utf-8") as f:
    document_text = f.read()
    print(f"Le texte lu est : {document_text}")

# 2. On prépare le payload
payload = {
    "text": document_text,
    "source": "wireguard2.md"
}

# 3. On envoie la requête POST à l'API Rust
print("Envoi en cours...")
response = requests.post("http://localhost:3000/api/ingest", json=payload)

# 4. On affiche le résultat
print(f"Status Code: {response.status_code}")
print(f"Réponse: {response.json()}")