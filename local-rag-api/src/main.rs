use axum::{
    routing::post,
    Router,
};
use qdrant_client::Qdrant;
// Nouveaux imports requis pour la création de la collection
use qdrant_client::qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder};
use reqwest::Client as HttpClient;
use std::sync::Arc;
use serde_json::json;
use tower_http::services::ServeDir;

mod models;
mod rag;

pub struct AppState {
    qdrant: Qdrant,
    http: HttpClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialisation du client Qdrant
    let qdrant = Qdrant::from_url("http://localhost:6334").build()?;
    
    // 2. Vérification et création de la collection au démarrage
    let collection_name = "rag";
    if !qdrant.collection_exists(collection_name).await? {
        println!("Création de la collection '{}' dans Qdrant...", collection_name);
        
        qdrant.create_collection(
            CreateCollectionBuilder::new(collection_name)
                // nomic-embed-text produit des vecteurs de taille 768
                // On utilise Cosine comme métrique de similarité
                .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine))
        ).await?;
        
        println!("Collection créée avec succès !");
    }

    // 3. Initialisation du client HTTP
    let http = HttpClient::new();
    let state = Arc::new(AppState { qdrant, http: http.clone() });

    // 4. Définition des routes
    let app = Router::new()
        .route("/api/ingest", post(rag::ingest_handler))
        .route("/api/ask", post(rag::ask_handler))
        .fallback_service(ServeDir::new("static")) 
        .with_state(state);

    let models_to_pull = vec!["nomic-embed-text", "mistral"];
    for model in models_to_pull {
        println!("Vérification/Téléchargement du modèle {} (cela peut prendre quelques minutes)...", model);
        let res = http.post("http://localhost:11434/api/pull")
            .json(&json!({ "name": model }))
            .send()
            .await?;
            
        if res.status().is_success() {
            println!("Modèle {} prêt !", model);
        } else {
            eprintln!("Erreur lors du téléchargement de {}", model);
        }
    }

    println!("Serveur démarré sur http://0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}