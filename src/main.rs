mod dweet;

use ethers::{
    providers::{Middleware, Provider},
    types::{Transaction, H256}, abi::AbiEncode,
};
use ethers_providers::{Http, ProviderExt, StreamExt};
use eyre::Result;


#[tokio::main]
async fn main() -> Result<()> {
    let provider = Provider::<Http>::connect("https://bsc-dataseed1.binance.org/").await;
    let mut stream = provider.watch_blocks().await?;
    // call digest on every block in stream
    while let Some(block) = stream.next().await {
        let block = provider.get_block_with_txs(block).await?.unwrap_or_default();
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
    if is_upgrade(tx).await {
        tokio::spawn(dispatch_upgrade(hash));
    }
    Ok(())
}

async fn is_upgrade(tx: Transaction) -> bool {
    // check if tx is a contract upgrade by checking first 4 bytes of data
    if tx.input.len() < 4 {
        return false;
    }
    // tx.input[0] == 169
    tx.input[0..4] == 0x99a88ec4_u32.to_be_bytes()
}

async fn dispatch_upgrade(hash: H256) -> Result<()> {
    let msg = format!("{}:upgrade", hash.encode_hex());
    // push to a dweet webhook
    let mut thing = dweet::Thing::new("bsc_board");
    thing.update(msg.as_str()).await.unwrap();
    // log
    println!("{}", msg);
    Ok(())
}