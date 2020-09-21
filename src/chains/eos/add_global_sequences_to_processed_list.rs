use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_database_utils::put_processed_tx_ids_in_db,
        eos_types::{
            RedeemInfo,
            ProcessedTxIds,
            GlobalSequences,
        },
    },
};

fn get_global_sequences_from_redeem_params(redeem_params: &[RedeemInfo]) -> GlobalSequences {
    redeem_params.iter().map(|params| params.global_sequence).collect::<GlobalSequences>()
}

pub fn add_tx_ids_to_processed_tx_ids<D>(
    db: &D,
    redeem_params: &[RedeemInfo],
    processed_tx_ids: &ProcessedTxIds,
) -> Result<()>
    where D: DatabaseInterface
{
    put_processed_tx_ids_in_db(
        db,
        &processed_tx_ids.clone().add_multi(&mut get_global_sequences_from_redeem_params(redeem_params))?
    )
}
