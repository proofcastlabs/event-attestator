mod account_for_fees;
mod extract_utxos_from_btc_txs;
mod filter_btc_txs_in_state;
mod get_eos_output;
mod redeem_info;
mod save_btc_utxos_to_db;
mod sign_transactions;
mod submit_eos_block;

// NOTE Needed so it can be in `state` - FIXME
#[cfg(test)]
pub(in crate::btc_on_eos) use self::redeem_info::BtcOnEosRedeemInfo;
pub(crate) use self::redeem_info::BtcOnEosRedeemInfos;
pub use self::submit_eos_block::submit_eos_block_to_core;
// NOTE: There are used in the EOS block reprocessor...
pub(in crate::btc_on_eos) use self::{
    account_for_fees::maybe_account_for_fees as maybe_account_for_peg_out_fees,
    extract_utxos_from_btc_txs::maybe_extract_btc_utxo_from_btc_tx_in_state,
    filter_btc_txs_in_state::maybe_filter_btc_txs_in_state,
    get_eos_output::get_eos_output,
    redeem_info::{maybe_filter_value_too_low_redeem_infos_in_state, maybe_parse_redeem_infos_and_put_in_state},
    save_btc_utxos_to_db::maybe_save_btc_utxos_to_db,
    sign_transactions::maybe_sign_txs_and_add_to_state,
};
