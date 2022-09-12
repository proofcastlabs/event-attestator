mod account_for_fees;
mod divert_to_safe_address;
mod eth_tx_info;
mod filter_eth_tx_infos;
mod get_btc_output_json;
mod parse_tx_infos;
mod sign_any_sender_transactions;
mod sign_normal_eth_transactions;
mod submit_btc_block;

// FIXME Currently used in chains::btc::btc_test_utils.
#[cfg(test)]
pub(crate) use self::eth_tx_info::BtcOnEthEthTxInfo;
// FIXME Currently required for use in `State`.
pub(crate) use self::eth_tx_info::BtcOnEthEthTxInfos;
pub use self::submit_btc_block::submit_btc_block_to_enclave;
// NOTE For use in debug block reprocessor.
pub(in crate::btc_on_eth) use self::{
    account_for_fees::{maybe_account_for_fees as maybe_account_for_minting_fees, subtract_fees_from_eth_tx_infos},
    divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
    filter_eth_tx_infos::maybe_filter_out_value_too_low_btc_on_eth_eth_tx_infos_in_state,
    get_btc_output_json::get_eth_signed_tx_info_from_eth_txs,
    parse_tx_infos::parse_eth_tx_infos_from_p2sh_deposits_and_add_to_state,
    sign_normal_eth_transactions::get_eth_signed_txs,
};
