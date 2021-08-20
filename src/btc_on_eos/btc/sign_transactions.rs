use crate::{
    btc_on_eos::btc::minting_params::BtcOnEosMintingParams,
    chains::{
        btc::{
            btc_chain_id::BtcChainId,
            btc_database_utils::{get_btc_canon_block_from_db, get_btc_chain_id_from_db},
            btc_metadata::ToMetadata,
            btc_state::BtcState,
        },
        eos::{
            eos_chain_id::EosChainId,
            eos_constants::MAX_BYTES_FOR_EOS_USER_DATA,
            eos_crypto::{
                eos_private_key::EosPrivateKey,
                eos_transaction::{get_signed_eos_ptoken_issue_tx, EosSignedTransaction, EosSignedTransactions},
            },
            eos_database_utils::{get_eos_account_name_string_from_db, get_eos_chain_id_from_db},
            eos_utils::get_eos_tx_expiration_timestamp_with_offset,
        },
    },
    metadata::metadata_protocol_id::MetadataProtocolId,
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_signed_eos_ptoken_issue_txs(
    ref_block_num: u16,
    ref_block_prefix: u32,
    chain_id: &EosChainId,
    pk: &EosPrivateKey,
    account: &str,
    minting_params: &BtcOnEosMintingParams,
    btc_chain_id: &BtcChainId,
) -> Result<EosSignedTransactions> {
    info!("✔ Signing {} txs...", minting_params.len());
    Ok(EosSignedTransactions::new(
        minting_params
            .iter()
            .enumerate()
            .map(|(i, params)| {
                get_signed_eos_ptoken_issue_tx(
                    ref_block_num,
                    ref_block_prefix,
                    &params.to,
                    &params.amount,
                    chain_id,
                    pk,
                    account,
                    get_eos_tx_expiration_timestamp_with_offset(i as u32)?,
                    params.maybe_to_metadata_bytes(
                        btc_chain_id,
                        MAX_BYTES_FOR_EOS_USER_DATA,
                        &MetadataProtocolId::Eos,
                    )?,
                )
            })
            .collect::<Result<Vec<EosSignedTransaction>>>()?,
    ))
}

pub fn maybe_sign_canon_block_txs_and_add_to_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe signing minting txs...");
    get_signed_eos_ptoken_issue_txs(
        state.get_eos_ref_block_num()?,
        state.get_eos_ref_block_prefix()?,
        &get_eos_chain_id_from_db(state.db)?,
        &EosPrivateKey::get_from_db(state.db)?,
        &get_eos_account_name_string_from_db(state.db)?,
        &get_btc_canon_block_from_db(state.db)?.get_eos_minting_params(),
        &get_btc_chain_id_from_db(state.db)?,
    )
    .and_then(|eos_signed_txs| state.add_eos_signed_txs(eos_signed_txs))
}
