use bitcoin::blockdata::transaction::Transaction as BtcTransaction;

use crate::{
    btc_on_eth::eth::redeem_info::BtcOnEthRedeemInfos,
    chains::{
        btc::{
            btc_crypto::btc_private_key::BtcPrivateKey,
            btc_database_utils::{get_btc_address_from_db, get_btc_fee_from_db, get_btc_private_key_from_db},
            btc_utils::get_pay_to_pub_key_hash_script,
            extract_utxos_from_p2pkh_txs::extract_utxos_from_p2pkh_tx,
            utxo_manager::utxo_database_utils::save_utxos_to_db,
        },
        eth::eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

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
    redeem_infos: &BtcOnEthRedeemInfos,
) -> Result<Vec<BtcTransaction>> {
    redeem_infos
        .filter_out_any_whose_value_is_too_low()
        .iter()
        .map(|redeem_info| redeem_info.to_btc_tx(db, fee, btc_address, btc_private_key))
        .map(|tx| extract_change_utxo_from_btc_tx_and_save_in_db(db, btc_address, tx?))
        .collect::<Result<Vec<_>>>()
}

pub fn maybe_create_btc_txs_and_add_to_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe creating BTC transaction(s) from redeem params...");
    let num_redeem_infos = state.btc_on_eth_redeem_infos.len();
    if num_redeem_infos == 0 {
        info!("✔ No `BtcOnEthRedeemInfos` in state ∴ not creating BTC txs!");
        Ok(state)
    } else {
        info!(
            "✔ {} `BtcOnEthRedeemInfos` in state ∴ creating BTC txs & extracting change outputs...",
            num_redeem_infos
        );
        to_btc_txs_whilst_extracting_change_outputs(
            &state.db,
            get_btc_fee_from_db(&state.db)?,
            &get_btc_address_from_db(&state.db)?,
            &get_btc_private_key_from_db(&state.db)?,
            &state.btc_on_eth_redeem_infos,
        )
        .and_then(|signed_txs| {
            #[cfg(feature = "debug")]
            {
                debug!("✔ Signed transactions: {:?}", signed_txs);
            }
            state.add_btc_transactions(signed_txs)
        })
    }
}
