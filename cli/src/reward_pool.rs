use anyhow::{Result, anyhow};
use config::Configuration;
use log::{info, error, warn};
use std::str::FromStr;
use signal_hook::{
    consts::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use crossbeam::sync::WaitGroup;
use std::sync::Arc;
// pub fn initialize_pool(matches: &clap::ArgMatches, config_file_path: String) -> Result<()> {
//     let mut config = Configuration::load_no_arc(config_file_path.as_str(), false)?;
//     vault_lib::initialize::initialize_vault(&mut config)?;
//     config.save(config_file_path.as_str(), false)?;
//     Ok(())
// }