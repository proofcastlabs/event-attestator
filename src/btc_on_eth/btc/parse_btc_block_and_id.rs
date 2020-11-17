use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::btc::{
        btc_state::BtcState,
        btc_types::BtcBlockAndId,
    },
};

pub fn parse_btc_block_and_id_and_put_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    BtcBlockAndId::from_json(state.get_btc_submission_json()?).and_then(|block| state.add_btc_block_and_id(block))
}
