use crate::{
    chains::btc::{btc_database_utils::BtcDbUtils, btc_state::BtcState},
    traits::DatabaseInterface,
    types::Result,
};

fn remove_tx_infos_from_canon_block<D: DatabaseInterface>(db_utils: &BtcDbUtils<D>) -> Result<()> {
    db_utils
        .get_btc_canon_block_from_db()
        .and_then(|canon_block| canon_block.remove_tx_infos())
        .and_then(|canon_block| db_utils.put_btc_canon_block_in_db(&canon_block))
}

pub fn remove_tx_infos_from_canon_block_and_return_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    info!("✔ Removing minting params from canon block...");
    remove_tx_infos_from_canon_block(&state.btc_db_utils).and(Ok(state))
}
