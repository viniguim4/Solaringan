use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use maplit::hashmap;
use futures::{Sink, Stream, channel::mpsc, SinkExt};
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::{prelude::*, tonic::{Status, transport::channel::ClientTlsConfig}};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub rpc: RpcConfig,
    pub grpc: GrpcConfig,
    pub wallets: WalletConfig,
    pub trade_settings: TradeSettings,
    pub dex: DexConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RpcConfig {
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrpcConfig {
    pub endpoint: String,
    pub x_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletConfig {
    pub targets: Vec<String>,
    pub copier: CopierWallet,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CopierWallet {
    pub private_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeSettings {
    pub min_entry: f64,
    pub max_entry: f64,
    pub slippage_tolerance: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DexConfig {
    pub raydium: RaydiumConfig,
    pub pumpfun: PumpfunConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RaydiumConfig {
    pub program_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PumpfunConfig {
    pub program_id: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Path::new("config/default.json");
        let config_str = std::fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;
        Ok(config)
    }

    pub async fn connect_grpc(&self) -> Result<GeyserGrpcClient<impl Interceptor>> {
        let url  = self.get_grpc_url();
        println!("Connecting to {}", url);
        let client = GeyserGrpcClient::build_from_shared(url).expect("Failed to build GeyserGrpcClient")
            .x_token(Some(self.grpc.x_token.clone())).expect("Failed to Auth x_token")
            .tls_config(ClientTlsConfig::new().with_native_roots()).expect("Error tls config")
            .connect()
            .await?;
        Ok(client)
    }

    fn build_request_monitor_wallet(&self, selector: u8) -> Result<SubscribeRequest> {
        let required_accounts = match selector {
            0 => self.dex.raydium.program_id.clone(),
            1 => self.dex.pumpfun.program_id.clone(),
            _ => return Err(anyhow!("Invalid selector"))   
        };
        let request = SubscribeRequest {
            slots: hashmap!{
                "".to_owned() => SubscribeRequestFilterSlots{
                    filter_by_commitment: Some(true),
                    interslot_updates: Some(true)
                }
            },
            transactions: hashmap!{
                "".to_owned() => SubscribeRequestFilterTransactions {
                    vote: Some(false),
                    failed: Some(false),
                    account_include: self.wallets.targets.clone(),
                    account_required:  vec![
                        required_accounts
                    ],
                    ..Default::default()
                }
            },
            commitment: Some(CommitmentLevel::Processed as i32),
            ..Default::default()
        };
        Ok(request)
    }

    pub async fn subscribe(&self, client: &mut GeyserGrpcClient<impl Interceptor>, selector: u8) -> Result<(
        impl Sink<SubscribeRequest, Error = mpsc::SendError>,
        impl Stream<Item = Result<SubscribeUpdate, Status>>,
    )> {
        let (mut tx, rx) = client.subscribe()
            .await
            .map_err(|e| anyhow!("Failed to subscribe to GeyserGrpcClient: {}", e))?;

        let request = self.build_request_monitor_wallet(selector)?;

        tx.send(request)
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        Ok((tx, rx))
    }

    pub async fn grpc_monitor(&self, selector: u8) -> Result<(
        impl Sink<SubscribeRequest, Error = mpsc::SendError>,
        impl Stream<Item = Result<SubscribeUpdate, Status>>,
    )> {
        let mut client = self.connect_grpc().await?;
        self.subscribe(&mut client, selector).await
    }

    pub fn get_grpc_url(&self) -> String {
        self.grpc.endpoint.clone()
    }
}
