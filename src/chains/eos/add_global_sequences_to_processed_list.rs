use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        eos_database_utils::put_processed_tx_ids_in_db,
        eos_types::{
            BtcOnEthRedeemInfos,
            ProcessedTxIds,
        },
    },
};

pub fn add_tx_ids_to_processed_tx_ids<D>(
    db: &D,
    redeem_infos: &BtcOnEthRedeemInfos,
    processed_tx_ids: &ProcessedTxIds,
) -> Result<()>
    where D: DatabaseInterface
{
    put_processed_tx_ids_in_db(db, &processed_tx_ids.clone().add_multi(&mut redeem_infos.get_global_sequences())?)
}

pub fn maybe_add_global_sequences_to_processed_list_and_return_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    match &state.redeem_infos.len() {
        0 => {
            info!("✔ No `global_sequences` to add to processed tx list!");
            Ok(state)
        }
        _ => {
            info!("✔ Adding `global_sequences` to processed tx list...");
            add_tx_ids_to_processed_tx_ids(&state.db, &state.redeem_infos, &state.processed_tx_ids).and(Ok(state))
        }
    }
}
