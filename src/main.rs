mod db;

use std::sync::Arc;

use crate::db::db::Db;
use axum::{routing::get, Router, extract::State, response::Html};
use ethers::{
    providers::{Middleware, Provider},
    types::Transaction, utils::hex::ToHex,
};
use ethers_providers::{Http, ProviderExt, StreamExt};
use eyre::Result;
use tokio::sync::RwLock;

const UPGRADE_SELECTOR: [u8; 4] = 0x99a88ec4_u32.to_be_bytes();

// blocklist set of addresses
const BLOCKLIST: [&str; 12] = [
    "0x3852f27ff39e66004b223501f9d24d480b6af3c9",
    "0x27310b0c0a54b0ea31efb02c6231498b59383f89",
    "0xb898d9900688eb9aeeb91b4328100343989434c6",
    "0x604676f0462085a165293f62f13b6cc73bce7fba",
    "0x846af2aa4e3a25a9edddcf738347feecc09bb976",
    "0x43a658230454fa6e769176b0147163f6298aab65",
    "0xb51d38fa0ceea0590f6cd168ae93f9983bc7b61c",
    "0x029be70984c83548a44f55ad72c24e2091555eb8",
    "0xce1d58c466c304a7e903ed68a515f65439d76d75",
    "0x41a7e0cd2c58bcf13e3f13df77d39a5816fbc9a5",
    "0x08db0f46c6ba4034f4104b117d2ec290f16fb6e9",
    "0xb54c9ab17722fa7a49bc9aa468264ddf82237305",
];

#[tokio::main]
async fn main() -> Result<()> {
    // initialize db
    let db = Arc::new(RwLock::new(Db::new()));

    // setup axum web server
    let r = Router::new().route("/", get(events)).with_state(db.clone());
    let server = axum::Server::bind(&"0.0.0.0:8080".parse().unwrap()).serve(r.into_make_service());

    println!("Listening on http://{}", server.local_addr());

    tokio::spawn(server);

    let provider = Provider::<Http>::connect("https://bsc-dataseed2.binance.org/").await;
    let mut stream = provider.watch_blocks().await?;
    // call digest on every block in stream
    while let Some(block) = stream.next().await {
        let block = provider
            .get_block_with_txs(block)
            .await?
            .unwrap_or_default();
        block.transactions.iter().for_each(|tx| {
            tokio::spawn(digest(tx.clone(), db.clone()));
        });
    }

    Ok(())
}

// events function for axum web server
async fn events(db: State<Arc<RwLock<Db>>>) -> Html<String> {
    let mut events = db.read().await.get_events_vec();
    events.reverse();
    let mut html = String::new();
    html.push_str("<html><body><table>");
    html.push_str("<tr><th>Event</th><th>Hash</th></tr>");
    for (hash, event) in events {
        html.push_str("<tr>");
        html.push_str(&format!("<td>{}</td>", event));
        html.push_str(&format!("<td><a href='https://bscscan.com/tx/0x{}'>{}</a></td>", hash.encode_hex::<String>(), hash.encode_hex::<String>()));
        html.push_str("</tr>");
    }
    html.push_str("</table></body></html>");
    html.into()
}

async fn digest(tx: Transaction, db: Arc<RwLock<Db>>) -> Result<()> {
    // check if tx is a contract upgrade
    if tx.input.len() < 4 {
        return Ok(());
    }
    if (tx.input[0..4] == UPGRADE_SELECTOR || tx.to == None)
        && !BLOCKLIST.contains(&format!("0x{0:020x}", tx.from).as_str())
    {
        if tx.input[0..4] == UPGRADE_SELECTOR {
            // if tx is a contract upgrade, dispatch immediately
            dispatch_event("upgrade", tx, db.clone()).await?;
        } else if tx.to == None {
            dispatch_event("deploy", tx, db.clone()).await?;
        }
    }
    Ok(())
}

async fn dispatch_event(event_type: &str, tx: Transaction, db: Arc<RwLock<Db>>) -> Result<()> {
    // store to db
    db.write().await.add_event(tx.hash, event_type.to_string());
    Ok(())
}