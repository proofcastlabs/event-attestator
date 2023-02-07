use crate::{
    chains::eth::{
        eth_constants::ETH_TAIL_LENGTH,
        eth_database_utils::EthDbUtilsExt,
        eth_submission_material::EthSubmissionMaterial,
    },
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

fn does_tail_block_require_updating<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    calculated_tail_block: &EthSubmissionMaterial,
) -> Result<bool> {
    info!("✔ Checking if ETH tail block needs updating...");
    db_utils
        .get_eth_tail_block_from_db()
        .and_then(|db_tail_block| Ok(db_tail_block.get_block_number()? < calculated_tail_block.get_block_number()?))
}

fn maybe_update_eth_tail_block_hash<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<()> {
    info!(
        "✔ Maybe updating {} tail block hash...",
        if db_utils.get_is_for_eth() { "ETH" } else { "EVM" }
    );
    let canon_to_tip_length = db_utils.get_eth_canon_to_tip_length_from_db()?;
    db_utils
        .get_eth_latest_block_from_db()
        .and_then(|latest_eth_block| {
            info!(
                "✔ Searching for tail block {} blocks back from tip...",
                canon_to_tip_length + ETH_TAIL_LENGTH
            );
            db_utils.maybe_get_nth_ancestor_eth_submission_material(
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
                match does_tail_block_require_updating(db_utils, &ancestor_block)? {
                    false => {
                        info!("✔ ETH tail block does not require updating");
                        Ok(())
                    },
                    true => {
                        info!("✔ Updating ETH tail block...");
                        db_utils.put_eth_tail_block_hash_in_db(&ancestor_block.get_block_hash()?)
                    },
                }
            },
        })
}

pub fn maybe_update_eth_tail_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_update_eth_tail_block_hash(&state.eth_db_utils).and(Ok(state))
}

pub fn maybe_update_evm_tail_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_update_eth_tail_block_hash(&state.evm_db_utils).and(Ok(state))
}
