mod account_for_fees;
mod divert_to_safe_address;
mod get_eos_output;
mod increment_eth_nonce;
mod redeem_info;
mod sign_normal_eth_txs;
mod submit_eos_block;

// FIXME Used in `State`
pub use submit_eos_block::submit_eos_block_to_core;

pub(crate) use self::redeem_info::Erc20OnEosRedeemInfos;
pub(in crate::erc20_on_eos) use self::{
    account_for_fees::{
        account_for_fees_in_redeem_infos_in_state,
        update_accrued_fees_in_dictionary_and_return_eos_state,
    },
    get_eos_output::{get_eth_signed_tx_info_from_eth_txs, EosOutput},
    increment_eth_nonce::maybe_increment_eth_nonce_in_db_and_return_eos_state,
    redeem_info::maybe_parse_redeem_infos_and_put_in_state,
    sign_normal_eth_txs::get_eth_signed_txs,
};
