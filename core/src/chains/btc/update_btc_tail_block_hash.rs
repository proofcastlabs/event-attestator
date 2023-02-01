use crate::{
    chains::btc::{btc_block::BtcBlockInDbFormat, btc_constants::BTC_TAIL_LENGTH, btc_database_utils::BtcDbUtils},
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

fn does_tail_block_require_updating<D: DatabaseInterface>(
    db_utils: &BtcDbUtils<D>,
    calculated_tail_block: &BtcBlockInDbFormat,
) -> Result<bool> {
    trace!("✔ Checking if BTC tail block needs updating...");
    db_utils
        .get_btc_tail_block_from_db()
        .map(|db_tail_block| db_tail_block.height < calculated_tail_block.height)
}

pub fn maybe_update_btc_tail_block_hash<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe updating BTC tail block hash...");
    let canon_to_tip_length = state.btc_db_utils.get_btc_canon_to_tip_length_from_db()?;
    state
        .btc_db_utils
        .get_btc_latest_block_from_db()
        .map(|latest_btc_block| {
            info!(
                "✔ Searching for tail block {} blocks back from tip...",
                canon_to_tip_length + BTC_TAIL_LENGTH,
            );
            state
                .btc_db_utils
                .maybe_get_nth_ancestor_btc_block_and_id(&latest_btc_block.id, canon_to_tip_length + BTC_TAIL_LENGTH)
        })
        .and_then(|maybe_ancester_block_and_id| match maybe_ancester_block_and_id {
            None => {
                info!(
                    "✔ No {}th ancestor block in db yet ∴ {}",
                    canon_to_tip_length, "not updating tail block hash!",
                );
                Ok(state)
            },
            Some(ancestor_block) => {
                info!("✔ {}th ancestor block found...", canon_to_tip_length + BTC_TAIL_LENGTH,);
                if does_tail_block_require_updating(&state.btc_db_utils, &ancestor_block)? {
                    info!("✔ Updating BTC tail block...");
                    state
                        .btc_db_utils
                        .put_btc_tail_block_hash_in_db(&ancestor_block.id)
                        .and(Ok(state))
                } else {
                    info!("✔ BTC tail block does not require updating");
                    Ok(state)
                }
            },
        })
}
