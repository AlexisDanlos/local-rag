use axum::{extract::State, Json};
use qdrant_client::qdrant::{PointStruct, QueryPointsBuilder, UpsertPointsBuilder};
use qdrant_client::Payload;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::AppState;

const OLLAMA_URL: &str = "http://localhost:11434";
const EMBEDDING_MODEL: &str = "nomic-embed-text";
const LLM_MODEL: &str = "mistral"; 
const COLLECTION_NAME: &str = "rag";

// --- HANDLERS ---

pub async fn ingest_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IngestRequest>,
) -> Json<serde_json::Value> {
    
    let chunks: Vec<&str> = payload.text.split("\n\n").collect();
    
    for chunk in chunks {
        if chunk.trim().is_empty() { continue; }

        let emb_req = OllamaEmbeddingRequest {
            model: EMBEDDING_MODEL.to_string(),
            prompt: chunk.to_string(),
        };

        if let Ok(res) = state.http.post(format!("{OLLAMA_URL}/api/embeddings"))
            .json(&emb_req)
            .send().await
        {
            if let Ok(emb_res) = res.json::<OllamaEmbeddingResponse>().await {
                
                let point = PointStruct::new(
                    Uuid::new_v4().to_string(),
                    emb_res.embedding,
                    Payload::try_from(serde_json::json!({
                        "text": chunk,
                        "source": payload.source
                    })).unwrap(),
                );

                // API Qdrant v1.19.0 : Utilisation du UpsertPointsBuilder
                match state.qdrant.upsert_points(
                    UpsertPointsBuilder::new(COLLECTION_NAME, vec![point])
                ).await {
                    Ok(_) => println!("Chunk inséré avec succès."),
                    Err(e) => {
                        eprintln!("Erreur lors de l'insertion dans Qdrant : {}", e);
                    }
                }
            }
        }
    }

    Json(serde_json::json!({"status": "success", "message": "Document ingéré"}))
}

pub async fn ask_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AskRequest>,
) -> Json<AskResponse> {

    let emb_req = OllamaEmbeddingRequest {
        model: EMBEDDING_MODEL.to_string(),
        prompt: payload.question.clone(),
    };
    
    let emb_res: OllamaEmbeddingResponse = state.http.post(format!("{OLLAMA_URL}/api/embeddings"))
        .json(&emb_req).send().await.unwrap().json().await.unwrap();

    // API Qdrant v1.19.0 : Remplacement de search_points par .query() et QueryPointsBuilder
    let search_res = state.qdrant.query(
        QueryPointsBuilder::new(COLLECTION_NAME)
            .query(emb_res.embedding)
            .limit(3)
            .with_payload(true)
    ).await.unwrap();
    
    let mut context_text = String::new();
    let mut sources = Vec::new();

    // API Qdrant v1.19.0 : Extraction propre du payload avec transformation en json
    for point in search_res.result {
        let mut payload = point.payload;
        
        // On récupère "text" et on le convertit de manière sécurisée
        if let Some(text_val) = payload.remove("text") {
            if let Some(text_str) = text_val.into_json().as_str() {
                context_text.push_str(&format!("- {}\n", text_str));
            }
        }
        
        // On récupère "source"
        if let Some(source_val) = payload.remove("source") {
            if let Some(source_str) = source_val.into_json().as_str() {
                sources.push(source_str.to_string());
            }
        }
    }

    let prompt = format!(
        "Tu es un expert technique francophone. Réponds à la question en te basant EXCLUSIVEMENT sur le contexte fourni ci-dessous. Si la réponse n'est pas dans le contexte, dis 'Je n'ai pas l'information'. Réponds uniquement en français.\n\nContexte:\n{}\n\nQuestion: {}\n\nRéponse:",
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