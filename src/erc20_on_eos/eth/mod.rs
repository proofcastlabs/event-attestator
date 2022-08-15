mod account_for_fees;
mod divert_to_safe_address;
mod get_output_json;
mod initialize_eth_core;
mod peg_in_info;
mod submit_eth_block;

// FIXME Used in `State`.
#[cfg(test)]
pub(in crate::erc20_on_eos) use self::peg_in_info::Erc20OnEosPegInInfo;
pub(crate) use self::peg_in_info::Erc20OnEosPegInInfos;
pub(in crate::erc20_on_eos) use self::{
    account_for_fees::{
        account_for_fees_in_peg_in_infos_in_state,
        update_accrued_fees_in_dictionary_and_return_eth_state,
    },
    get_output_json::get_output_json,
    peg_in_info::{
        filter_out_zero_value_peg_ins_from_state,
        filter_submission_material_for_peg_in_events_in_state,
        maybe_sign_eos_txs_and_add_to_eth_state,
    },
};
pub use self::{initialize_eth_core::maybe_initialize_eth_core, submit_eth_block::submit_eth_block_to_core};
