//! Lance le relais : `bs-relais [adresse]` (par défaut 0.0.0.0:8787). Routes : `/salon`
//! (WebSocket) et `/etat` (JSON).

#[tokio::main]
async fn main() {
    let adresse = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8787".into());
    let ecoute = tokio::net::TcpListener::bind(&adresse)
        .await
        .expect("adresse d'écoute");
    println!(
        "relais Block Survivors sur {adresse} (protocole {})",
        bs_protocole::VERSION_PROTOCOLE
    );
    bs_relais::servir(ecoute).await.expect("serveur");
}
