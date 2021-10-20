use config::{Configuration};
use anyhow::{Result, anyhow};

pub fn new_config(_matches: &clap::ArgMatches, config_file_path: String) -> Result<()> {
    Configuration::new(config_file_path.as_str(), false)?;
    Ok(())
}

pub fn export_as_json(_matches: &clap::ArgMatches, config_file_path: String) -> Result<()>  {
    let mut config = Configuration::load_no_arc(config_file_path.as_str(), false)?;
    config.http_rpc_url = "".to_string();
    config.ws_rpc_url = "".to_string();
    let name_parts: Vec<&str> = config_file_path.split(".").collect();
    let mut name = String::new();
    name.push_str(name_parts[0]);
    name.push_str(".json");
    config
        .save(name.as_str(), true)?;
    Ok(())
}
