use bitcoin::blockdata::transaction::Transaction as BtcTransaction;

use crate::{btc_constants::MAX_NUM_OUTPUTS, btc_types::BtcTransactions};

pub fn maybe_filter_out_btc_txs_with_too_many_outputs(txs: &[BtcTransaction]) -> BtcTransactions {
    info!("✔ Filtering out BTC transactions with > {} outputs...", MAX_NUM_OUTPUTS);
    debug!("Num tx before: {}", txs.len());
    let filtered_txs = txs
        .iter()
        .filter(|tx| {
            if tx.output.len() > MAX_NUM_OUTPUTS {
                info!(
                    "✘ Filtering out BTC tx because it has > than {} outputs!\n {:?}",
                    MAX_NUM_OUTPUTS, tx
                );
                false
            } else {
                true
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    debug!("Num tx after: {}", filtered_txs.len());
    filtered_txs
}
