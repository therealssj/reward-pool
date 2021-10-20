use anchor_client::{Client, Cluster};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use simplelog::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, read_keypair_file};
use std::{fs::File};
use std::{fs};
use std::str::FromStr;
use std::sync::Arc;
/// main configuration object
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    pub key_path: String,
    pub db_url: String,
    pub log_file: String,
    pub debug_log: bool,
    pub http_rpc_url: String,
    pub ws_rpc_url: String,
    pub analytics_api: String,
    pub pools: Vec<Pool>,
    pub programs: Programs,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Programs {
    pub reward_pool: Program,
}
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RewardPool {
    pub initialized: bool,
}

/// a deployed program we interact with
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Program {
    /// the program id
    pub id: String,
    /// path to the anchor generated idl file
    pub idl_path: String,
}
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pool {
    pub name: String,
    pub account: String,
    pub account_nonce: u8,
    pub authority: String,
    pub x_token_pool_vault: String,
    pub staking_mint: String,
    pub staking_vault: String,
    pub reward_a_mint: String,
    pub reward_a_vault: String,
    pub reward_b_mint: String,
    pub reward_b_vault: String,
    pub reward_duration: u64,
    pub reward_end: u64,
    pub reward_a_rate: u64,
    pub reward_b_rate: u64,
}


impl Programs {
    pub fn reward_pool_id(&self) -> Pubkey {
        Pubkey::from_str(self.reward_pool.id.as_str()).unwrap()
    }
}

impl Configuration {
    pub fn new(path: &str, as_json: bool) -> Result<()> {
        let config = Configuration::default();
        config.save(path, as_json)
    }
    pub fn save(&self, path: &str, as_json: bool) -> Result<()> {
        let data = if as_json {
            serde_json::to_string_pretty(&self)?
        } else {
            serde_yaml::to_string(&self)?
        };
        fs::write(path, data).expect("failed to write to file");
        Ok(())
    }
    pub fn load_no_arc(path: &str, from_json: bool) -> Result<Configuration> {
        let data = fs::read(path).expect("failed to read file");
        let config: Configuration = if from_json {
            serde_json::from_slice(data.as_slice())?
        } else {
            serde_yaml::from_slice(data.as_slice())?
        };
        Ok(config)
    }
    pub fn load(path: &str, from_json: bool) -> Result<Arc<Configuration>> {
        let data = fs::read(path).expect("failed to read file");
        let config: Configuration = if from_json {
            serde_json::from_slice(data.as_slice())?
        } else {
            serde_yaml::from_slice(data.as_slice())?
        };
        Ok(Arc::new(config))
    }
    // returns the primary rpc provider
    pub fn anchor_client(&self) -> Client {
        let payer = read_keypair_file(self.key_path.clone()).expect("failed to read keypair file");
        let cluster = Cluster::Custom(
            self.http_rpc_url.clone(),
            self.ws_rpc_url.clone(),
        );
        Client::new_with_options(cluster, payer, CommitmentConfig::confirmed())
    }
    pub fn rpc_client(&self) -> RpcClient {
        RpcClient::new_with_timeout_and_commitment(
            self.http_rpc_url.to_string(),
            std::time::Duration::from_secs(30),
            CommitmentConfig::confirmed(),
        )
    }
    pub fn payer(&self) -> Keypair {
        read_keypair_file(self.key_path.clone()).expect("failed to read keypair file")
    }

    pub fn get_pool_config(&self, pool_name: String) -> Result<Pool> {
        for pool in self.pools.iter() {
            if pool.name.eq(&pool_name) {
                return Ok(pool.clone())
            }
        }

        Err(anyhow!("no pool config found for given pool name"))
    }
    /// if file_log is true, log to both file and stdout
    /// otherwise just log to stdout
    pub fn init_log(&self, file_log: bool) -> Result<()> {
        if !file_log {
            if self.debug_log {
                TermLogger::init(
                    LevelFilter::Debug,
                    ConfigBuilder::new()
                        .set_location_level(LevelFilter::Debug)
                        .build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                )?;
                return Ok(());
            } else {
                TermLogger::init(
                    LevelFilter::Info,
                    ConfigBuilder::new()
                        .set_location_level(LevelFilter::Error)
                        .build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                )?;
                return Ok(());
            }
        }
        if self.debug_log {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    ConfigBuilder::new()
                        .set_location_level(LevelFilter::Debug)
                        .build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Debug,
                    ConfigBuilder::new()
                        .set_location_level(LevelFilter::Debug)
                        .build(),
                    File::create(self.log_file.as_str()).unwrap(),
                ),
            ])?;
        } else {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Info,
                    ConfigBuilder::new()
                        .set_location_level(LevelFilter::Error)
                        .build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Info,
                    ConfigBuilder::new()
                        .set_location_level(LevelFilter::Error)
                        .build(),
                    File::create(self.log_file.as_str()).unwrap(),
                ),
            ])?;
        }
        Ok(())
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            key_path: "~/.config/solana/id.json".to_string(),
            db_url: "postgres://postgres:necc@postgres/kek".to_string(),
            log_file: "template.log".to_string(),
            debug_log: false,
            http_rpc_url: "https://solana-api.projectserum.com".to_string(),
            ws_rpc_url: "ws://solana-api.projectserum.com".to_string(),
            analytics_api: "".to_string(),
            programs: Programs{
                reward_pool: Program{
                    id: "FoNqK2xudK7TfKjPFxpzAcTaU2Wwyt81znT4RjJBLFQp".to_string(),
                    idl_path: "".to_string(),
                },
            },
            pools: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}