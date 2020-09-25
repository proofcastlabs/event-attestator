use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::{
        eos::eos_erc20_account_names::EosErc20AccountNames,
        eth::{
            eth_state::EthState,
            eth_database_utils::get_eth_canon_block_from_db,
        },
    },
};

pub fn maybe_parse_peg_in_info_and_add_to_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe parsing `erc20-on-eos` peg-in infos...");
    get_eth_canon_block_from_db(&state.db)
        .and_then(|submission_material| {
            match submission_material.receipts.is_empty() {
                true => {
                    info!("✔ No receipts in canon block ∴ no info to parse!");
                    Ok(state)
                }
                false => {
                    info!("✔ {} receipts in canon block ∴ parsing info...", submission_material.block.number);
                    EosErc20AccountNames::get_from_db(&state.db)
                        .and_then(|account_names| submission_material.get_erc20_on_eos_peg_in_infos(&account_names))
                        .and_then(|peg_in_infos| state.add_erc20_on_eos_peg_in_infos(peg_in_infos))
                }
            }
        })
}
