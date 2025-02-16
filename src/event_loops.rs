use log::{info};
use tokio::time::{sleep, Duration};
use anyhow::{Result, Context};
use futures::StreamExt;
use solana_sdk::signature::Signature;
use solana_transaction_status::TransactionWithStatusMeta;
use yellowstone_grpc_proto::prelude::{subscribe_update::UpdateOneof, SubscribeUpdateTransactionInfo};
use yellowstone_grpc_proto::convert_from;
use crate::config::Config;
use crate::parser::{decode_pumpfun_txn, decode_raydium_txn};

pub async fn start_monitor_loop(config: Config, selector: u8) -> Result<()> {
    // Spawn the monitoring task
    tokio::spawn(async move {
        loop {
            match monitor_wallet(&config, selector).await {
                Ok(_) => {
                    match selector {
                        0 => info!("RAYDIUM monitor loop ended unexpectedly"),
                        1 => info!("PUMPFUN monitor loop ended unexpectedly"),
                        _ => {
                            info!("Not supposed to get here")
                        }
                    }
                }
                Err(e) => {
                    match selector {
                        0 => info!("RAYDIUM monitor reconnecting due to: {}", e),
                        1 => info!("PUMPFUN monitor reconnecting due to: {}", e),
                        _ => {
                            info!("Not supposed to get here")
                        }
                    }
                }
            }
            
            // Wait before attempting to reconnect
            info!("Attempting to reconnect in 5 seconds...");
            sleep(Duration::from_secs(5)).await;
        }
    });

    Ok(())
}

async fn monitor_wallet(config: &Config, selector: u8) -> Result<()> {
    // Initialize wallet monitor
    let (_tx, mut grpc_rx) = config.grpc_monitor(selector).await?;
    match selector {
        0 => info!("RAYDIUM monitor initialized successfully"),
        1 => info!("PUMPFUN monitor initialized successfully"),
        _ => {
            info!("Not supposed to get here")
        }
    }

    while let Some(message) = grpc_rx.next().await {
        match message {
            Ok(info) => {
                match info.update_oneof {
                    Some(UpdateOneof::Transaction(tx)) => {
                        let slot = tx.slot;
                        let transaction = match tx.transaction {
                            Some(tx) => tx,
                            None => {
                                continue;
                            }
                        };
                        let pretty_tx = create_pretty_transaction(transaction)?;
                        match selector {
                            0 => {
                                info!("RAYDIUM Transaction at slot {}: {:#?}", slot, &pretty_tx);
                                let _decoded_tx = decode_raydium_txn(&pretty_tx.tx)?;
                                
                            },
                            1 => {
                                info!("PUMPFUN Transaction at slot {}: {:#?}", slot, &pretty_tx);
                                let _decoded_tx = decode_pumpfun_txn(&pretty_tx.tx)?;
                                
                            },
                            _ => {
                                info!("Not supposed to get here ");
                            }
                        }
                    }
                    Some(UpdateOneof::Slot(_slot)) => {
                        // Uncomment if you want to log slots
                        //info!("Slot: {:#?}", slot);
                    }
                    _ => {
                        continue;
                    }
                }
            }
            Err(e) => {
                match selector {
                    0 => info!("RAYDIUM monitor Disconnected from gRPC client: {}", e),
                    1 => info!("PUMPFUN monitor Disconnected from gRPC client: {}", e),
                    _ => {
                        info!("Not supposed to get here")
                    }
                }
                return Err(anyhow::anyhow!("gRPC connection error: {}", e));
            }
        }
    }

    // If we get here, the stream has ended
    Err(anyhow::anyhow!("Stream ended unexpectedly"))
}


fn create_pretty_transaction(tx: SubscribeUpdateTransactionInfo) -> Result<PrettyTransaction> {
    Ok(PrettyTransaction {
        signature: Signature::try_from(tx.signature.as_slice()).context("invalid signature")?.to_string(),
        is_vote: tx.is_vote,
        tx: convert_from::create_tx_with_meta(tx)
            .map_err(|error| anyhow::anyhow!(error))
            .context("invalid tx with meta")?
    })
}

#[derive(Debug)]
pub struct PrettyTransaction {
    signature: String,
    is_vote: bool,
    tx: TransactionWithStatusMeta
}

