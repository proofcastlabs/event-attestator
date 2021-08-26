use crate::{
    chains::eth::{
        eth_constants::ETH_TAIL_LENGTH,
        eth_database_utils::EthDatabaseUtils,
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    traits::DatabaseInterface,
    types::Result,
};

fn does_tail_block_require_updating<D: DatabaseInterface>(
    eth_db_utils: &EthDatabaseUtils<D>,
    calculated_tail_block: &EthSubmissionMaterial,
) -> Result<bool> {
    info!("✔ Checking if ETH tail block needs updating...");
    eth_db_utils
        .get_eth_tail_block_from_db()
        .and_then(|db_tail_block| Ok(db_tail_block.get_block_number()? < calculated_tail_block.get_block_number()?))
}

pub fn maybe_update_eth_tail_block_hash<D: DatabaseInterface>(eth_db_utils: &EthDatabaseUtils<D>) -> Result<()> {
    info!("✔ Maybe updating ETH tail block hash...");
    let canon_to_tip_length = eth_db_utils.get_eth_canon_to_tip_length_from_db()?;
    eth_db_utils
        .get_eth_latest_block_from_db()
        .and_then(|latest_eth_block| {
            info!(
                "✔ Searching for tail block {} blocks back from tip...",
                canon_to_tip_length + ETH_TAIL_LENGTH
            );
            eth_db_utils.maybe_get_nth_ancestor_eth_submission_material(
                &latest_eth_block.get_block_hash()?,
                canon_to_tip_length + ETH_TAIL_LENGTH,
            )
        })
        .and_then(|maybe_ancester_block_and_id| match maybe_ancester_block_and_id {
            None => {
                info!(
                    "✔ No {}th ancestor block in db ∴ {}",
                    canon_to_tip_length, "not updating tail block hash!"
                );
                Ok(())
            },
            Some(ancestor_block) => {
                info!("✔ {}th ancestor block found...", canon_to_tip_length + ETH_TAIL_LENGTH);
                match does_tail_block_require_updating(eth_db_utils, &ancestor_block)? {
                    false => {
                        info!("✔ ETH tail block does not require updating");
                        Ok(())
                    },
                    true => {
                        info!("✔ Updating ETH tail block...");
                        eth_db_utils.put_eth_tail_block_hash_in_db(&ancestor_block.get_block_hash()?)
                    },
                }
            },
        })
}

pub fn maybe_update_eth_tail_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe updating ETH tail block hash...");
    maybe_update_eth_tail_block_hash(&state.eth_db_utils).and(Ok(state))
}
