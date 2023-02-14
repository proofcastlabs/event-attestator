use bitcoin::blockdata::transaction::Transaction as BtcTransaction;
use common::{
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_btc::{
    extract_utxos_from_p2pkh_tx,
    get_pay_to_pub_key_hash_script,
    save_utxos_to_db,
    BtcDbUtils,
    BtcPrivateKey,
    BtcTransactions,
};
use common_eth::EthState;

use crate::eth::btc_tx_info::BtcOnEthBtcTxInfos;

pub fn extract_change_utxo_from_btc_tx_and_save_in_db<D: DatabaseInterface>(
    db: &D,
    btc_address: &str,
    btc_tx: BtcTransaction,
) -> Result<BtcTransaction> {
    get_pay_to_pub_key_hash_script(btc_address)
        .map(|target_script| extract_utxos_from_p2pkh_tx(&target_script, &btc_tx))
        .map(|ref utxos| save_utxos_to_db(db, utxos))
        .map(|_| btc_tx)
}

fn to_btc_txs_whilst_extracting_change_outputs<D: DatabaseInterface>(
    db: &D,
    fee: u64,
    btc_address: &str,
    btc_private_key: &BtcPrivateKey,
    btc_tx_infos: &BtcOnEthBtcTxInfos,
) -> Result<BtcTransactions> {
    Ok(BtcTransactions::new(
        btc_tx_infos
            .filter_out_any_whose_value_is_too_low()
            .iter()
            .map(|btc_tx_info| {
                debug!("Signing BTC tx...");
                debug!("    To: {}", btc_tx_info.recipient);
                debug!("  From: {}", btc_tx_info.from);
                debug!("Amount: {} satoshis", btc_tx_info.amount_in_satoshis);
                debug!("   Fee: {} sats/byte", fee);
                btc_tx_info.to_btc_tx(db, fee, btc_address, btc_private_key)
            })
            .map(|tx| extract_change_utxo_from_btc_tx_and_save_in_db(db, btc_address, tx?))
            .collect::<Result<Vec<_>>>()?,
    ))
}

pub fn maybe_create_btc_txs_and_add_to_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ No `BtcOnEthBtcTxInfos` in state ∴ not creating BTC txs!");
        Ok(state)
    } else {
        info!("✔ `BtcOnEthBtcTxInfos` in state ∴ creating BTC txs & extracting change outputs...");
        let btc_db_utils = BtcDbUtils::new(state.db);
        to_btc_txs_whilst_extracting_change_outputs(
            state.db,
            btc_db_utils.get_btc_fee_from_db()?,
            &btc_db_utils.get_btc_address_from_db()?,
            &btc_db_utils.get_btc_private_key_from_db()?,
            &BtcOnEthBtcTxInfos::from_bytes(&state.tx_infos)?,
        )
        .and_then(|signed_txs| {
            debug!("✔ Signed transactions: {:?}", signed_txs);
            signed_txs.to_bytes()
        })
        .and_then(|bytes| state.add_signed_txs(bytes))
    }
}
