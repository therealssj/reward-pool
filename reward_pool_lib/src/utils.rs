use solana_sdk::pubkey::Pubkey;

pub fn derive_pool_signer(
    program_id: &Pubkey,
    pool: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            pool.as_ref(),
        ],
        program_id,
    )
}