use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::{
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
};

pub fn parse_eth_submission_material_and_put_in_state<D>(
    block_json: &str,
    state: EthState<D>,
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing ETH block & receipts...");
    EthSubmissionMaterial::from_str(&block_json).and_then(|result| state.add_eth_submission_material(result))
}
