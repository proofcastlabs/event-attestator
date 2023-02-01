use crate::{
    btc_on_eth::btc::eth_tx_info::BtcOnEthEthTxInfo,
    chains::eth::{
        any_sender::relay_transaction::RelayTransaction,
        eth_database_utils::EthDbUtilsExt,
        eth_types::{AnySenderSigningParams, RelayTransactions},
    },
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_any_sender_signed_txs(
    signing_params: &AnySenderSigningParams,
    eth_tx_infos: &[BtcOnEthEthTxInfo],
) -> Result<RelayTransactions> {
    trace!("✔ Getting AnySender signed transactions...");
    eth_tx_infos
        .iter()
        .enumerate()
        .map(|(i, eth_tx_info)| {
            info!(
                "✔ Signing AnySender tx for amount: {}, to address: {}",
                eth_tx_info.amount, eth_tx_info.destination_address,
            );

            let any_sender_nonce = signing_params.any_sender_nonce + i as u64;

            RelayTransaction::new_mint_by_proxy_tx(
                &signing_params.chain_id,
                signing_params.public_eth_address,
                eth_tx_info.amount,
                any_sender_nonce,
                &signing_params.eth_private_key,
                signing_params.erc777_proxy_address,
                eth_tx_info.destination_address,
            )
        })
        .collect::<Result<RelayTransactions>>()
}

pub fn maybe_sign_any_sender_canon_block_txs_and_add_to_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    if !state.use_any_sender_tx_type() {
        info!("✔ Using normal ETH therefore not signing AnySender transactions!");
        return Ok(state);
    }
    info!("✔ Maybe signing AnySender txs...");
    get_any_sender_signed_txs(
        &state.eth_db_utils.get_any_sender_signing_params_from_db()?,
        &state.btc_db_utils.get_btc_canon_block_from_db()?.get_eth_tx_infos(),
    )
    .and_then(|signed_txs| {
        debug!("✔ Signed AnySender transactions: {:?}", signed_txs);
        state.add_any_sender_signed_txs(signed_txs)
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{hashes::Hash, util::address::Address as BtcAddress, Txid};

    use super::*;
    use crate::{
        btc_on_eth::{btc::eth_tx_info::BtcOnEthEthTxInfo, utils::convert_satoshis_to_wei},
        chains::{
            btc::btc_test_utils::SAMPLE_TARGET_BTC_ADDRESS,
            eth::{
                eth_chain_id::EthChainId,
                eth_test_utils::{get_sample_eth_address, get_sample_eth_private_key},
                eth_types::EthAddress,
            },
        },
    };

    #[test]
    fn should_get_any_sender_signatures() {
        let signing_params = AnySenderSigningParams {
            chain_id: EthChainId::Mainnet,
            any_sender_nonce: 0,
            eth_private_key: get_sample_eth_private_key(),
            public_eth_address: get_sample_eth_address(),
            erc777_proxy_address: get_sample_eth_address(),
        };
        let originating_address = BtcAddress::from_str(SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        let recipient_1 = EthAddress::from_slice(&hex::decode("789e39e46117DFaF50A1B53A98C7ab64750f9Ba3").unwrap());
        let recipient_2 = EthAddress::from_slice(&hex::decode("9360a5C047e8Eb44647f17672638c3bB8e2B8a53").unwrap());
        let user_data = None;
        let eth_token_address = EthAddress::default();
        let eth_tx_infos = vec![
            BtcOnEthEthTxInfo::new(
                convert_satoshis_to_wei(1337),
                hex::encode(recipient_1),
                Txid::from_hash(Hash::hash(&[0xc0])),
                originating_address.clone(),
                user_data.clone(),
                &eth_token_address,
            )
            .unwrap(),
            BtcOnEthEthTxInfo::new(
                convert_satoshis_to_wei(666),
                hex::encode(recipient_2),
                Txid::from_hash(Hash::hash(&[0xc0])),
                originating_address,
                user_data,
                &eth_token_address,
            )
            .unwrap(),
        ];
        let result = get_any_sender_signed_txs(&signing_params, &eth_tx_infos).unwrap();
        assert_eq!(result.len(), eth_tx_infos.len());
    }
}
