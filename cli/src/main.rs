mod config;
mod reward_pool;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;
use anyhow::{Result, anyhow};
use tokio;
use clap::{App, Arg, SubCommand};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("template-cli")
        .version("0.0.1")
        .author("reward pool")
        .about("template cli for rust projects")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("sets the config file")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("vault")
                .about("vault management commands")
                .subcommands(vec![
                    SubCommand::with_name("new")
                        .about("create a new vault account")
                        .arg(
                            Arg::with_name("farm-name")
                                .short("f")
                                .long("farm-name")
                                .help("name of the farm to generate a config for")
                                .value_name("NAME")
                        )
                        .arg(
                            Arg::with_name("create-tokens")
                                .short("c")
                                .long("create-tokens")
                                .help("enable token account creation")
                                .takes_value(false)
                        ),
                    SubCommand::with_name("initialize")
                        .about("used to initialize the vault program state account"),
                    SubCommand::with_name("gen-vault-config")
                        .about("generates a tempalte vault config")
                        .arg(
                            Arg::with_name("farm-name")
                                .short("f")
                                .long("farm-name")
                                .help("name of the farm to generate a config for")
                                .value_name("NAME")
                        )
                ])
        )
        .get_matches();
    let config_file_path = get_config_or_default(&matches);
    process_matches(&matches, config_file_path).await?;
    Ok(())
}

// returns the value of the config file argument or the default
fn get_config_or_default(matches: &clap::ArgMatches) -> String {
    matches
        .value_of("config")
        .unwrap_or("config.yaml")
        .to_string()
}

async fn process_matches<'a>(matches: &clap::ArgMatches<'a>, config_file_path: String) -> Result<()> {
    match matches.subcommand() {
        // ("vault", Some(vault_command)) => match vault_command.subcommand() {
        //     ("initialize", Some(initialize_vault)) => {
        //         vault::initialize_vault(initialize_vault, config_file_path)
        //     }
        //     ("new", Some(new_vault)) => {
        //         vault::new_vault(new_vault, config_file_path)
        //     }
        //     _ => invalid_subcommand("vault"),
        // }
        _ => invalid_command(),
    }
}

fn invalid_subcommand(command_group: &str) -> Result<()> {
    Err(anyhow!("invalid command found for group {}", command_group))
}

fn invalid_command() -> Result<()> {
    Err(anyhow!("invalid command found"))
}