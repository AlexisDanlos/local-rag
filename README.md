# 🦀 Local RAG : Moteur d'Inférence et Base de Connaissances Sécurisée

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8.x-blue.svg)](https://github.com/tokio-rs/axum)
[![Qdrant](https://img.shields.io/badge/Qdrant-1.19-red.svg)](https://qdrant.tech/)
[![Ollama](https://img.shields.io/badge/Ollama-Local_LLM-white.svg)](https://ollama.com/)

Ce projet est une implémentation complète d'un système **RAG (Retrieval-Augmented Generation) 100% local et déconnecté**. Il permet d'interroger des documents techniques de manière sémantique, sans qu'aucune donnée ne quitte la machine, garantissant une confidentialité totale.

L'architecture est pensée pour maximiser les performances matérielles grand public (optimisé pour une enveloppe de 8 Go de VRAM, type NVIDIA RTX 3070).

---

## 🏗️ Architecture et Stack Technique

Le projet s'articule autour d'une séparation stricte des responsabilités :

*   **Backend (Rust / Axum / Tokio) :** Orchestrateur asynchrone hautes performances. Il gère l'ingestion, le découpage des documents, et le formatage des requêtes.
*   **Base de Données Vectorielle (Qdrant) :** Moteur de recherche sémantique exécuté dans Docker.
*   **Moteur d'Inférence (Ollama) :** Exécution locale des modèles accélérée par CUDA (GPU Passthrough via Docker).
    *   *Embedding :* `nomic-embed-text` (Vecteurs de 768 dimensions).
    *   *LLM :* `mistral` (Quantifié, optimisé pour la précision technique et le respect strict du contexte).
*   **Frontend (Vanilla HTML/JS/Tailwind) :** Interface Zero-Dependency servie statiquement par le backend Rust.

---

## ✨ Fonctionnalités Clés

- **Auto-Provisioning :** Au démarrage, le backend Rust vérifie et crée automatiquement les collections vectorielles requises, et provisionne les modèles d'IA via l'API d'Ollama.
- **Zéro Dépendance Front-End :** Pas de Node.js, pas de build process. L'interface est un client léger ultra-réactif.
- **Sécurité et Typage Fort :** L'utilisation de Rust garantit l'absence de fuites mémoire et une gestion rigoureuse des erreurs lors de la communication inter-services.

---

## 🚀 Démarrage Rapide

### Prérequis
- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- Le Toolkit [NVIDIA Container](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html) (pour l'accélération matérielle)
- [Rust & Cargo](https://rustup.rs/)

### Installation

**1. Démarrer l'infrastructure (Bases de données et Inférence)**
```bash
docker-compose up -d
```

**2. Lancer le Backend Rust**
```bash
cargo run
```
*Note : Au premier lancement, le backend téléchargera automatiquement les modèles IA dans le conteneur Ollama (cela peut prendre quelques minutes selon votre connexion).*

**3. Accéder à l'interface**
Ouvrez votre navigateur et rendez-vous sur : `http://localhost:3000`

---

## 📡 Endpoints API

Si vous préférez utiliser le projet en mode *Headless* (via Postman ou cURL) :

### Ingestion d'un document
```http
POST /api/ingest
Content-Type: application/json

{
  "text": "Le contenu de votre documentation technique...",
  "source": "nom_du_fichier.md"
}
```

### Interrogation (Chat)
```http
POST /api/ask
Content-Type: application/json

{
  "question": "Comment configurer le protocole réseau ?"
}
```

---

## 🎯 Pourquoi ce projet ?

L'objectif était de démontrer la mise en place d'un pipeline MLOps complet, de la gestion de l'infrastructure sous-jacente (GPU passthrough) jusqu'à l'exposition d'une API backend robuste. Le choix de Rust s'est imposé pour sa fiabilité d'exécution et sa capacité à traiter des flux de données asynchrones sans compromis sur la consommation des ressources.

---
**Auteur :** Alexis Danlos  
*Développé dans le cadre d'une exploration des architectures IA décentralisées.*