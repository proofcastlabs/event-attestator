use crate::{
    chains::eos::{
        eos_constants::{EOS_MAX_EXPIRATION_SECS, MEMO, PEOS_ACCOUNT_PERMISSION_LEVEL},
        eos_crypto::{
            eos_private_key::EosPrivateKey,
            eos_transaction::{get_unsigned_eos_minting_tx, sign_peos_transaction},
        },
        eos_types::EosSignedTransaction,
    },
    types::Result,
};

pub fn get_signed_tx(
    ref_block_num: u16,
    ref_block_prefix: u32,
    to: &str,
    amount: &str,
    chain_id: &str,
    private_key: &EosPrivateKey,
    account_name: &str,
) -> Result<EosSignedTransaction> {
    info!("✔ Signing eos minting tx for {} to {}...", &amount, &to);
    get_unsigned_eos_minting_tx(
        to,
        account_name,
        MEMO,
        account_name,
        amount,
        ref_block_num,
        ref_block_prefix,
        EOS_MAX_EXPIRATION_SECS,
        PEOS_ACCOUNT_PERMISSION_LEVEL,
    )
    .and_then(|unsigned_tx| sign_peos_transaction(to, amount, chain_id, private_key, &unsigned_tx))
}
