use crate::{
    btc_on_eos::btc::eos_tx_info::BtcOnEosEosTxInfos,
    chains::{
        btc::{btc_chain_id::BtcChainId, btc_metadata::ToMetadata},
        eos::{
            eos_chain_id::EosChainId,
            eos_constants::MAX_BYTES_FOR_EOS_USER_DATA,
            eos_crypto::{
                eos_private_key::EosPrivateKey,
                eos_transaction::{get_signed_eos_ptoken_issue_tx, EosSignedTransaction, EosSignedTransactions},
            },
            eos_utils::get_eos_tx_expiration_timestamp_with_offset,
        },
    },
    metadata::metadata_protocol_id::MetadataProtocolId,
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_signed_eos_ptoken_issue_txs(
    ref_block_num: u16,
    ref_block_prefix: u32,
    chain_id: &EosChainId,
    pk: &EosPrivateKey,
    account: &str,
    eos_tx_infos: &BtcOnEosEosTxInfos,
    btc_chain_id: &BtcChainId,
) -> Result<EosSignedTransactions> {
    info!("✔ Signing {} txs...", eos_tx_infos.len());
    Ok(EosSignedTransactions::new(
        eos_tx_infos
            .iter()
            .enumerate()
            .map(|(i, params)| {
                get_signed_eos_ptoken_issue_tx(
                    ref_block_num,
                    ref_block_prefix,
                    &params.destination_address,
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
    info!("✔ Maybe signing EOS txs...");
    get_signed_eos_ptoken_issue_txs(
        state.get_eos_ref_block_num()?,
        state.get_eos_ref_block_prefix()?,
        &state.eos_db_utils.get_eos_chain_id_from_db()?,
        &EosPrivateKey::get_from_db(state.db)?,
        &state.eos_db_utils.get_eos_account_name_string_from_db()?,
        &state
            .btc_db_utils
            .get_btc_canon_block_from_db()?
            .get_btc_on_eos_eos_tx_infos(),
        &state.btc_db_utils.get_btc_chain_id_from_db()?,
    )
    .and_then(|eos_signed_txs| state.add_eos_signed_txs(eos_signed_txs))
}
