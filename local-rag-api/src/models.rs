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