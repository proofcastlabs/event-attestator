mod account_for_fees;
mod btc_tx_info;
mod extract_utxos_from_btc_txs;
mod filter_btc_txs_in_state;
mod get_eos_output;
mod initialize_eos_core;
mod save_btc_utxos_to_db;
mod sign_transactions;
mod submit_eos_block;

// NOTE Needed so it can be in `state` - FIXME
#[cfg(test)]
pub(super) use self::btc_tx_info::BtcOnEosBtcTxInfo;
pub(crate) use self::btc_tx_info::BtcOnEosBtcTxInfos;
// NOTE: There are used in the EOS block reprocessor...
pub(super) use self::{
    account_for_fees::maybe_account_for_fees as maybe_account_for_peg_out_fees,
    btc_tx_info::{maybe_filter_value_too_low_btc_tx_infos_in_state, maybe_parse_btc_tx_infos_and_put_in_state},
    extract_utxos_from_btc_txs::maybe_extract_btc_utxo_from_btc_tx_in_state,
    filter_btc_txs_in_state::maybe_filter_btc_txs_in_state,
    get_eos_output::get_eos_output,
    save_btc_utxos_to_db::maybe_save_btc_utxos_to_db,
    sign_transactions::maybe_sign_txs_and_add_to_state,
};
pub use self::{initialize_eos_core::maybe_initialize_eos_core, submit_eos_block::submit_eos_block_to_core};
