use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_database_utils::put_processed_tx_ids_in_db,
        eos_types::{
            RedeemInfos,
            ProcessedTxIds,
        },
    },
};

pub fn add_tx_ids_to_processed_tx_ids<D>(
    db: &D,
    redeem_infos: &RedeemInfos,
    processed_tx_ids: &ProcessedTxIds,
) -> Result<()>
    where D: DatabaseInterface
{
    put_processed_tx_ids_in_db(db, &processed_tx_ids.clone().add_multi(&mut redeem_infos.get_global_sequences())?)
}
