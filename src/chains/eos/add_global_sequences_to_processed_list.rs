use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::eos_database_utils::put_processed_tx_ids_in_db,
    btc_on_eos::eos::eos_types::{
        RedeemParams,
        ProcessedTxIds,
        GlobalSequences,
    },
};

fn get_global_sequences_from_redeem_params(redeem_params: &[RedeemParams]) -> GlobalSequences {
    redeem_params.iter().map(|params| params.global_sequence).collect::<GlobalSequences>()
}

pub fn add_tx_ids_to_processed_tx_ids<D>(
    db: &D,
    redeem_params: &[RedeemParams],
    processed_tx_ids: &ProcessedTxIds,
) -> Result<()>
    where D: DatabaseInterface
{
    put_processed_tx_ids_in_db(
        db,
        &processed_tx_ids.clone().add_multi(&mut get_global_sequences_from_redeem_params(redeem_params))?
    )
}
