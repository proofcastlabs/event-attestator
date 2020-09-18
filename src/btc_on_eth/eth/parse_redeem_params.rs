use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::eth::eth_database_utils::get_eth_canon_block_from_db,
};

pub fn maybe_parse_redeem_params_and_add_to_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe parsing redeem params...");
    get_eth_canon_block_from_db(&state.db)
        .and_then(|block_and_receipts| {
            match block_and_receipts.receipts.is_empty() {
                true => {
                    info!("✔ No receipts in canon block ∴ no params to parse!");
                    Ok(state)
                }
                false => {
                    info!("✔ Receipts in canon block #{}∴ parsing params...", block_and_receipts.block.number);
                    block_and_receipts.get_redeem_params().and_then(|params| state.add_redeem_params(params))
                }
            }
        })
}
