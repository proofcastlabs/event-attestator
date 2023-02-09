use common::{state::BtcState, traits::DatabaseInterface, types::Result};
use common_eth::{AnySenderSigningParams, EthDbUtils, EthDbUtilsExt, RelayTransaction, RelayTransactions};

use crate::btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos};

pub fn get_any_sender_signed_txs(
    signing_params: &AnySenderSigningParams,
    eth_tx_infos: &[BtcOnEthEthTxInfo],
) -> Result<RelayTransactions> {
    trace!("✔ Getting AnySender signed transactions...");
    Ok(RelayTransactions(
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
            .collect::<Result<Vec<_>>>()?,
    ))
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
        &EthDbUtils::new(state.db).get_any_sender_signing_params_from_db()?,
        &BtcOnEthEthTxInfos::from_bytes(&state.btc_db_utils.get_btc_canon_block_from_db()?.get_tx_info_bytes())?,
    )
    .and_then(|signed_txs| {
        debug!("✔ Signed AnySender transactions: {:?}", signed_txs);
        state.add_any_sender_signed_txs(signed_txs.to_bytes()?)
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{hashes::Hash, util::address::Address as BtcAddress, Txid};
    use common::{chains::btc::btc_test_utils::SAMPLE_TARGET_BTC_ADDRESS, EthChainId};
    use common_eth::test_utils::{get_sample_eth_address, get_sample_eth_private_key};
    use ethereum_types::Address as EthAddress;

    use super::*;
    use crate::{btc::eth_tx_info::BtcOnEthEthTxInfo, utils::convert_satoshis_to_wei};

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
