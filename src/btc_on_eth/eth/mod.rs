mod account_for_fees;
mod add_erc777_contract_address;
mod create_btc_transactions;
mod filter_receipts_in_state;
mod get_eth_output_json;
mod increment_btc_nonce;
mod initialize_eth_core;
mod redeem_info;
mod submit_eth_block;

#[cfg(test)]
pub(in crate::btc_on_eth) use self::redeem_info::BtcOnEthRedeemInfo;
pub(crate) use self::redeem_info::BtcOnEthRedeemInfos; // FIXME Used in `State`
pub(in crate::btc_on_eth) use self::{
    account_for_fees::{maybe_account_for_fees, subtract_fees_from_redeem_infos},
    create_btc_transactions::{extract_change_utxo_from_btc_tx_and_save_in_db, maybe_create_btc_txs_and_add_to_state},
    filter_receipts_in_state::filter_receipts_for_btc_on_eth_redeem_events_in_state,
    get_eth_output_json::{get_btc_signed_tx_info_from_btc_txs, EthOutput},
    increment_btc_nonce::maybe_increment_btc_nonce_in_db_and_return_state,
};
pub use self::{
    add_erc777_contract_address::maybe_add_erc777_contract_address,
    initialize_eth_core::maybe_initialize_eth_enclave,
    submit_eth_block::submit_eth_block_to_enclave,
};
