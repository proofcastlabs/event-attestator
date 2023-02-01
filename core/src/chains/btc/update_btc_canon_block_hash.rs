use crate::{
    chains::btc::{btc_block::BtcBlockInDbFormat, btc_database_utils::BtcDbUtils, btc_state::BtcState},
    traits::DatabaseInterface,
    types::Result,
};

fn does_canon_block_require_updating<D: DatabaseInterface>(
    db_utils: &BtcDbUtils<D>,
    calculated_canon_block: &BtcBlockInDbFormat,
) -> Result<bool> {
    info!("✔ Checking if BTC canon block needs updating...");
    db_utils
        .get_btc_canon_block_from_db()
        .map(|db_canon_block_and_receipts| db_canon_block_and_receipts.height < calculated_canon_block.height)
}

pub fn maybe_update_btc_canon_block_hash<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe updating BTC canon block hash...");
    let canon_to_tip_length = state.btc_db_utils.get_btc_canon_to_tip_length_from_db()?;
    state
        .btc_db_utils
        .get_btc_latest_block_from_db()
        .map(|latest_btc_block| {
            state
                .btc_db_utils
                .maybe_get_nth_ancestor_btc_block_and_id(&latest_btc_block.id, canon_to_tip_length)
        })
        .and_then(|maybe_ancester_block_and_id| match maybe_ancester_block_and_id {
            None => {
                info!(
                    "✔ No {}th ancestor block in db yet ∴ {}",
                    canon_to_tip_length, "not updating canon block hash!",
                );
                Ok(state)
            },
            Some(ancestor_block) => {
                info!("✔ {}th ancestor block found...", canon_to_tip_length,);
                if does_canon_block_require_updating(&state.btc_db_utils, &ancestor_block)? {
                    info!("✔ Updating BTC canon block...");
                    state
                        .btc_db_utils
                        .put_btc_canon_block_hash_in_db(&ancestor_block.id)
                        .and(Ok(state))
                } else {
                    info!("✔ BTC canon block does not require updating");
                    Ok(state)
                }
            },
        })
}
