use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_database_utils::put_processed_tx_ids_in_db,
        eos_types::{
            RedeemParams,
            ProcessedTxIds,
        },
    },
};

fn get_tx_ids_from_redeem_params(
    redeem_params: &Vec<RedeemParams>
) -> Vec<String> {
    redeem_params
        .iter()
        .map(|params| params.originating_tx_id.to_string())
        .collect::<Vec<String>>()
}

fn add_tx_ids_to_processed_tx_ids<D>(
    db: &D,
    redeem_params: &Vec<RedeemParams>,
    processed_tx_ids: &ProcessedTxIds,
) -> Result<()>
    where D: DatabaseInterface
{
    put_processed_tx_ids_in_db(
        db,
        &processed_tx_ids
            .clone()
            .add_multi(&mut get_tx_ids_from_redeem_params(redeem_params))?
    )
}

pub fn maybe_add_tx_ids_to_processed_tx_ids<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    match &state.redeem_params.len() {
        0 => {
            info!("✔ No txs to add to processed tx list!");
            Ok(state)
        }
        _ => {
            info!("✔ Adding txs to processed tx list...");
            add_tx_ids_to_processed_tx_ids(
                &state.db,
                &state.redeem_params,
                &state.processed_tx_ids,
            )
                .map(|_| state)
        }
    }
}
