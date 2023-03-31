mod dweet;

use ethers::{
    providers::{Middleware, Provider},
    types::Transaction, abi::AbiEncode,
};
use ethers_providers::{Http, ProviderExt, StreamExt};
use eyre::Result;

const UPGRADE_SELECTOR: [u8; 4] = 0x99a88ec4_u32.to_be_bytes();

// blocklist set of addresses
const BLOCKLIST: [&str; 8] = [
    "0x3852f27ff39e66004b223501f9d24d480b6af3c9",
    "0x27310b0c0a54b0ea31efb02c6231498b59383f89",
    "0xb898d9900688eb9aeeb91b4328100343989434c6",
    "0x604676f0462085a165293f62f13b6cc73bce7fba",
    "0x846af2aa4e3a25a9edddcf738347feecc09bb976",
    "0x43a658230454fa6e769176b0147163f6298aab65",
    "0xb51d38fa0ceea0590f6cd168ae93f9983bc7b61c",
    "0x029be70984c83548a44f55ad72c24e2091555eb8",
];

#[tokio::main]
async fn main() -> Result<()> {
    let provider = Provider::<Http>::connect("https://bsc-dataseed2.binance.org/").await;
    let mut stream = provider.watch_blocks().await?;
    // call digest on every block in stream
    while let Some(block) = stream.next().await {
        let block = provider
            .get_block_with_txs(block)
            .await?
            .unwrap_or_default();
        block.transactions.iter().for_each(|tx| {
            tokio::spawn(digest(tx.clone()));
        });
    }

    Ok(())
}

async fn digest(tx: Transaction) -> Result<()> {
    // do something with tx
    let hash = tx.hash;
    // check if tx is a contract upgrade
    if tx.input.len() < 4 {
        return Ok(());
    }
    if tx.input[0..4] == UPGRADE_SELECTOR || tx.to == None || !BLOCKLIST.contains(&tx.from.to_string().as_str()){
        if tx.input[0..4] == UPGRADE_SELECTOR {
            // if tx is a contract upgrade, dispatch immediately
            dispatch_upgrade(format!("upgrade:{}", hash.encode_hex())).await?;
        } else if tx.to == None{
            dispatch_upgrade(format!("deploy:{}", hash.encode_hex())).await?;
        }
    }
    Ok(())
}

async fn dispatch_upgrade(msg: String) -> Result<()> {
    // push to a dweet webhook
    let mut thing = dweet::Thing::new("bsc_board");
    thing.update(msg.as_str()).await.unwrap();
    // log
    println!("{}", msg);
    Ok(())
}