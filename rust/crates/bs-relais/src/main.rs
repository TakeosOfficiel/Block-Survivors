//! Lance le relais : `bs-relais [adresse]` (par défaut 0.0.0.0:8787). Routes : `/salon`
//! (WebSocket) et `/etat` (JSON). Derrière Cloudflare (tunnel conseillé, docs/RELAIS.md),
//! l'adresse des joueurs vient de l'en-tête `CF-Connecting-IP` ; exposé directement,
//! lancer avec `BS_RELAIS_DIRECT=1` (sinon cet en-tête, falsifiable, serait cru).

#[tokio::main]
async fn main() {
    let adresse = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8787".into());
    let direct = std::env::var("BS_RELAIS_DIRECT").is_ok_and(|v| v == "1");
    let ecoute = tokio::net::TcpListener::bind(&adresse)
        .await
        .expect("adresse d'écoute");
    println!(
        "relais Block Survivors sur {adresse} (protocole {}, {})",
        bs_protocole::VERSION_PROTOCOLE,
        if direct {
            "exposé directement"
        } else {
            "derrière Cloudflare"
        }
    );
    let reglages = bs_relais::Reglages {
        cloudflare: !direct,
        ..Default::default()
    };
    bs_relais::servir_avec(ecoute, reglages)
        .await
        .expect("serveur");
}
