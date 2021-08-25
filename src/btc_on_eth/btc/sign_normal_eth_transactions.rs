use crate::{
    btc_on_eth::btc::minting_params::BtcOnEthMintingParamStruct,
    chains::{
        btc::{
            btc_chain_id::BtcChainId,
            btc_database_utils::{get_btc_canon_block_from_db, get_btc_chain_id_from_db},
            btc_metadata::ToMetadata,
            btc_state::BtcState,
        },
        eth::{
            eth_constants::MAX_BYTES_FOR_ETH_USER_DATA,
            eth_crypto::eth_transaction::{get_signed_minting_tx, EthTransaction, EthTransactions},
            eth_types::EthSigningParams,
        },
    },
    metadata::metadata_protocol_id::MetadataProtocolId,
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_eth_signed_txs(
    signing_params: &EthSigningParams,
    minting_params: &[BtcOnEthMintingParamStruct],
    btc_chain_id: &BtcChainId,
) -> Result<EthTransactions> {
    trace!("✔ Getting ETH signed transactions...");
    Ok(EthTransactions::new(
        minting_params
            .iter()
            .enumerate()
            .map(|(i, minting_param_struct)| {
                info!(
                    "✔ Signing ETH tx for amount: {}, to address: {}",
                    minting_param_struct.amount, minting_param_struct.eth_address,
                );
                get_signed_minting_tx(
                    &minting_param_struct.amount,
                    signing_params.eth_account_nonce + i as u64,
                    &signing_params.chain_id,
                    signing_params.smart_contract_address,
                    signing_params.gas_price,
                    &minting_param_struct.eth_address,
                    &signing_params.eth_private_key,
                    minting_param_struct.maybe_to_metadata_bytes(
                        btc_chain_id,
                        MAX_BYTES_FOR_ETH_USER_DATA,
                        &MetadataProtocolId::Ethereum,
                    )?,
                    None,
                )
            })
            .collect::<Result<Vec<EthTransaction>>>()?,
    ))
}

pub fn maybe_sign_normal_canon_block_txs_and_add_to_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    if state.use_any_sender_tx_type() {
        info!("✔ Using AnySender therefore not signing normal ETH transactions!");
        return Ok(state);
    }
    info!("✔ Maybe signing normal ETH txs...");
    get_eth_signed_txs(
        &state.eth_db_utils.get_signing_params_from_db()?,
        &get_btc_canon_block_from_db(state.db)?.get_eth_minting_params(),
    )
    .and_then(|signed_txs| {
        #[cfg(feature = "debug")]
        {
            debug!("✔ Signed transactions: {:?}", signed_txs);
        }
        state.add_eth_signed_txs(signed_txs)
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{hashes::Hash, util::address::Address as BtcAddress, Txid};

    use super::*;
    use crate::{
        btc_on_eth::utils::convert_satoshis_to_wei,
        chains::{
            btc::btc_test_utils::SAMPLE_TARGET_BTC_ADDRESS,
            eth::{
                eth_chain_id::EthChainId,
                eth_database_utils_redux::EthDatabaseUtils,
                eth_test_utils::{get_sample_eth_address, get_sample_eth_private_key},
                eth_types::EthAddress,
            },
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_get_eth_signing_params() {
        let nonce = 6;
        let chain_id = EthChainId::Mainnet;
        let db = get_test_database();
        let eth_db_utils = EthDatabaseUtils::new(&db);
        let gas_price = 20_000_000_000;
        let contract_address = get_sample_eth_address();
        let eth_private_key = get_sample_eth_private_key();
        eth_db_utils
            .put_btc_on_eth_smart_contract_address_in_db(&contract_address)
            .unwrap();
        eth_db_utils.put_eth_chain_id_in_db(&chain_id).unwrap();
        eth_db_utils.put_eth_gas_price_in_db(gas_price).unwrap();
        eth_db_utils.put_eth_account_nonce_in_db(nonce).unwrap();
        eth_db_utils.put_eth_private_key_in_db(&eth_private_key).unwrap();
        let result = eth_db_utils.get_signing_params_from_db().unwrap();
        assert!(
            result.chain_id == chain_id
                && result.gas_price == gas_price
                && result.eth_account_nonce == nonce
                && result.eth_private_key == eth_private_key
                && result.smart_contract_address == contract_address
        );
    }

    #[test]
    fn should_get_eth_signatures() {
        let signing_params = EthSigningParams {
            chain_id: EthChainId::Mainnet,
            eth_account_nonce: 0,
            gas_price: 20_000_000_000,
            eth_private_key: get_sample_eth_private_key(),
            smart_contract_address: get_sample_eth_address(),
        };
        let originating_address = BtcAddress::from_str(SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        let recipient_1 = EthAddress::from_slice(&hex::decode("789e39e46117DFaF50A1B53A98C7ab64750f9Ba3").unwrap());
        let recipient_2 = EthAddress::from_slice(&hex::decode("9360a5C047e8Eb44647f17672638c3bB8e2B8a53").unwrap());
        let user_data = None;
        let minting_params = vec![
            BtcOnEthMintingParamStruct::new(
                convert_satoshis_to_wei(1337),
                hex::encode(recipient_1),
                Txid::from_hash(Hash::hash(&[0xc0])),
                originating_address.clone(),
                user_data.clone(),
            )
            .unwrap(),
            BtcOnEthMintingParamStruct::new(
                convert_satoshis_to_wei(666),
                hex::encode(recipient_2),
                Txid::from_hash(Hash::hash(&[0xc0])),
                originating_address,
                user_data,
            )
            .unwrap(),
        ];
        let btc_chain_id = BtcChainId::Bitcoin;
        let result = get_eth_signed_txs(&signing_params, &minting_params, &btc_chain_id).unwrap();
        assert_eq!(result.len(), minting_params.len());
    }
}
