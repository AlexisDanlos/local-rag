# Structure du Projet RAG Local : Rust + Axum + Qdrant + Ollama

Ce document contient l'architecture de base et le code nécessaire pour initialiser le projet.

## 1. `docker-compose.yml`
L'orchestration est primordiale. Ce fichier déploie la base vectorielle Qdrant et Ollama, en exposant le GPU (NVIDIA RTX) pour accélérer l'inférence.

```yaml
version: '3.8'

services:
  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
      - "6334:6334"
    volumes:
      - qdrant_data:/qdrant/storage

  ollama:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
    volumes:
      - ollama_data:/root/.ollama
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]

volumes:
  qdrant_data:
  ollama_data:
```

## 2. `Cargo.toml`
Les dépendances nécessaires pour un backend web performant et asynchrone en Rust.

```toml
[package]
name = "local_rag_api"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
qdrant-client = "1.7"
anyhow = "1.0"
thiserror = "1.0"
uuid = { version = "1.7", features = ["v4", "fast-rng"] }
```

## 3. `src/main.rs`
Le point d'entrée. On configure le routeur Axum et on injecte le client Qdrant et un client HTTP dans l'état de l'application.

```rust
use axum::{
    routing::post,
    Router,
    extract::State,
    Json,
};
use qdrant_client::prelude::*;
use reqwest::Client as HttpClient;
use std::sync::Arc;

mod models;
mod rag;

// L'état partagé de l'application
pub struct AppState {
    qdrant: QdrantClient,
    http: HttpClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialisation du client Qdrant
    let qdrant = QdrantClient::from_url("http://localhost:6334").build()?;
    
    // Initialisation du client HTTP pour Ollama
    let http = HttpClient::new();

    let state = Arc::new(AppState { qdrant, http });

    // Définition des routes de l'API
    let app = Router::new()
        .route("/api/ingest", post(rag::ingest_handler))
        .route("/api/ask", post(rag::ask_handler))
        .with_state(state);

    println!("Serveur démarré sur http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## 4. `src/models.rs`
Définition des structures de données (DTOs) pour communiquer avec les clients et Ollama.

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct IngestRequest {
    pub text: String,
    pub source: String,
}

#[derive(Deserialize)]
pub struct AskRequest {
    pub question: String,
}

#[derive(Serialize)]
pub struct AskResponse {
    pub answer: String,
    pub sources: Vec<String>,
}

// Structures pour l'API locale d'Ollama
#[derive(Serialize)]
pub struct OllamaEmbeddingRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct OllamaEmbeddingResponse {
    pub embedding: Vec<f32>,
}

#[derive(Serialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Deserialize)]
pub struct OllamaGenerateResponse {
    pub response: String,
}
```

## 5. `src/rag.rs`
Le cœur de la logique métier : création des embeddings, interaction avec Qdrant, et construction du prompt enrichi.

```rust
use axum::{extract::State, Json};
use qdrant_client::qdrant::{PointStruct, SearchPoints};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

const OLLAMA_URL: &str = "http://localhost:11434";
const EMBEDDING_MODEL: &str = "nomic-embed-text";
const LLM_MODEL: &str = "deepseek-r1:1.5b"; // Modèle très léger adapté pour une exécution ultra-rapide
const COLLECTION_NAME: &str = "research_papers";

// --- HANDLERS ---

pub async fn ingest_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IngestRequest>,
) -> Json<serde_json::Value> {
    
    // 1. Découpage basique (à améliorer avec un vrai Text Splitter plus tard)
    let chunks: Vec<&str> = payload.text.split("\n\n").collect();
    
    for chunk in chunks {
        if chunk.trim().is_empty() { continue; }

        // 2. Générer l'embedding via Ollama
        let emb_req = OllamaEmbeddingRequest {
            model: EMBEDDING_MODEL.to_string(),
            prompt: chunk.to_string(),
        };

        if let Ok(res) = state.http.post(format!("{OLLAMA_URL}/api/embeddings"))
            .json(&emb_req)
            .send().await
        {
            if let Ok(emb_res) = res.json::<OllamaEmbeddingResponse>().await {
                // 3. Sauvegarder dans Qdrant
                let point = PointStruct::new(
                    Uuid::new_v4().to_string(),
                    emb_res.embedding,
                    serde_json::json!({
                        "text": chunk,
                        "source": payload.source
                    }).try_into().unwrap(),
                );

                let _ = state.qdrant.upsert_points(COLLECTION_NAME, None, vec![point], None).await;
            }
        }
    }

    Json(serde_json::json!({"status": "success", "message": "Document ingéré"}))
}

pub async fn ask_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AskRequest>,
) -> Json<AskResponse> {

    // 1. Vectoriser la question
    let emb_req = OllamaEmbeddingRequest {
        model: EMBEDDING_MODEL.to_string(),
        prompt: payload.question.clone(),
    };
    
    let emb_res: OllamaEmbeddingResponse = state.http.post(format!("{OLLAMA_URL}/api/embeddings"))
        .json(&emb_req).send().await.unwrap().json().await.unwrap();

    // 2. Chercher les chunks pertinents dans Qdrant
    let search_req = SearchPoints {
        collection_name: COLLECTION_NAME.to_string(),
        vector: emb_res.embedding,
        limit: 3,
        with_payload: Some(true.into()),
        ..Default::default()
    };

    let search_res = state.qdrant.search_points(&search_req).await.unwrap();
    
    let mut context_text = String::new();
    let mut sources = Vec::new();

    for point in search_res.result {
        if let Some(payload) = point.payload {
            if let Some(text_val) = payload.get("text") {
                context_text.push_str(&format!("- {}\n", text_val.to_string()));
            }
            if let Some(source_val) = payload.get("source") {
                sources.push(source_val.to_string());
            }
        }
    }

    // 3. Construire le prompt et interroger le LLM
    let prompt = format!(
        "Tu es un assistant technique. Réponds à la question en te basant UNIQUEMENT sur le contexte fourni.\n\nContexte:\n{}\n\nQuestion: {}\n\nRéponse:",
        context_text, payload.question
    );

    let llm_req = OllamaGenerateRequest {
        model: LLM_MODEL.to_string(),
        prompt,
        stream: false,
    };

    let llm_res: OllamaGenerateResponse = state.http.post(format!("{OLLAMA_URL}/api/generate"))
        .json(&llm_req).send().await.unwrap().json().await.unwrap();

    Json(AskResponse {
        answer: llm_res.response,
        sources,
    })
}