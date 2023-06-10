use common::{traits::DatabaseInterface, types::Result};

use crate::{
    bitcoin_crate_alias::blockdata::transaction::Transaction as BtcTransaction,
    btc_database_utils::BtcDbUtils,
    btc_utils::get_pay_to_pub_key_hash_script,
    extract_utxos_from_p2pkh_txs::extract_utxos_from_p2pkh_txs,
    utxo_manager::BtcUtxosAndValues,
};

pub fn extract_btc_utxo_from_btc_tx<D: DatabaseInterface>(
    db_utils: &BtcDbUtils<D>,
    signed_txs: &[BtcTransaction],
) -> Result<BtcUtxosAndValues> {
    db_utils
        .get_btc_address_from_db()
        .and_then(|address| get_pay_to_pub_key_hash_script(&address))
        .map(|target_script| extract_utxos_from_p2pkh_txs(&target_script, signed_txs))
}
