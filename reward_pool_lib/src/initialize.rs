use config::{Configuration, Pool};
use reward_pool;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use std::str::FromStr;
use spl_token;
use log::{info, error};
use anyhow::{Result, anyhow};
use crate::utils::derive_pool_signer;

pub fn initialize_pool(
    config: &mut Configuration,
    pool_name: &String,
    create_token_accounts: bool,
    config_file_path: String
) -> Result<()> {
    let client = config.anchor_client();
    let program = client.program(config.programs.reward_pool_id());
    let payer = config.payer();

    let pool_config = config.get_pool_config(pool_name.clone())?;
    let pool = Keypair::new();
    let (pool_signer, pool_nonce) = derive_pool_signer(&config.programs.reward_pool_id(), &pool.pubkey());

    let pool_accont_size = std::mem::size_of::<reward_pool::Pool>() + 8; // 8 for discriminator
    let rent = program.rpc().get_minimum_balance_for_rent_exemption(pool_accont_size)?;
    let instruction = solana_sdk::system_instruction::create_account(
        &payer.pubkey(),
        &pool.pubkey(),
        rent,
        pool_accont_size as u64,
        &payer.pubkey()
    );
    let reward_duration = pool_config.reward_duration;
    let staking_mint = Pubkey::from_str(&pool_config.staking_mint)?;
    let reward_a_mint = Pubkey::from_str(&pool_config.reward_a_mint)?;
    let reward_b_mint = Pubkey::from_str(&pool_config.reward_b_mint)?;

    let staking_vault = spl_associated_token_account::get_associated_token_address(
        &pool_signer,
        &staking_mint,
    );
    info!("staking vault {}", staking_vault);

    let reward_a_vault = spl_associated_token_account::get_associated_token_address(
        &pool_signer,
        &reward_a_mint,
    );
    info!("reward a vault {}", reward_a_vault);

    let reward_b_vault = spl_associated_token_account::get_associated_token_address(
        &pool_signer,
        &reward_b_mint,
    );
    info!("reward b vault {}", reward_b_vault);

    if create_token_accounts {
        let mut instructions = vec![];
        instructions.push(spl_associated_token_account::create_associated_token_account(
            &payer.pubkey(),
            &pool_signer,
            &staking_mint,
        ));
        instructions.push(spl_associated_token_account::create_associated_token_account(
            &payer.pubkey(),
            &pool_signer,
            &reward_a_mint,
        ));
        instructions.push(spl_associated_token_account::create_associated_token_account(
            &payer.pubkey(),
            &pool_signer,
            &reward_b_mint,
        ));
        let (blockhash, _fee_calc) = program.rpc().get_recent_blockhash()?;

        let mut tx = Transaction::new_with_payer(
            &instructions[..],
            Some(&payer.pubkey()),
        );
        tx.sign(&[&payer], blockhash);
        let sig = program.rpc().send_and_confirm_transaction(&tx)?;
        info!("sent token account creation tx {}", sig);
    }
    let sig = program
        .request()
        .args(reward_pool::instruction::InitializePool{
            pool_nonce,
            reward_duration
        })
        .accounts(reward_pool::accounts::InitializePool{
            authority: payer.pubkey(),
            staking_mint,
            staking_vault,
            reward_a_mint,
            reward_a_vault,
            reward_b_mint,
            reward_b_vault,
            pool_signer,
            pool: pool.pubkey(),
            token_program: spl_token::id(),
        })
        .instruction(
            instruction
        )
        .send()?;

    Ok(())
}