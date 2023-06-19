use common::{
    traits::DatabaseInterface,
    types::{NoneError, Result},
};
use common_btc::{BtcState, DepositInfoHashMap};
use common_eth::{EthDbUtils, EthDbUtilsExt};
use ethereum_types::Address as EthAddress;

use crate::{
    bitcoin_crate_alias::{
        blockdata::transaction::Transaction as BtcTransaction,
        network::constants::Network as BtcNetwork,
        util::address::Address as BtcAddress,
    },
    btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos},
    utils::convert_satoshis_to_wei,
};

impl BtcOnEthEthTxInfos {
    fn from_btc_tx(
        tx: &BtcTransaction,
        deposit_info: &DepositInfoHashMap,
        network: BtcNetwork,
        eth_token_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Parsing eth tx infos from single `P2SH` transaction...");
        Ok(Self::new(
            tx.output
                .iter()
                .filter(|tx_out| tx_out.script_pubkey.is_p2sh())
                .map(|tx_out| match BtcAddress::from_script(&tx_out.script_pubkey, network) {
                    Err(_) => {
                        info!("✘ Could not derive BTC address from tx: {:?}", tx);
                        (tx_out, None)
                    },
                    Ok(address) => {
                        info!("✔ BTC address extracted from `tx_out`: {}", address);
                        (tx_out, Some(address))
                    },
                })
                .filter(|(_, maybe_address)| maybe_address.is_some())
                .map(|(tx_out, address)| {
                    match deposit_info.get(&address.clone().ok_or(NoneError("Could not unwrap BTC address!"))?) {
                        None => {
                            info!(
                                "✘ BTC address {} not in deposit list!",
                                address.ok_or(NoneError("Could not unwrap BTC address!"))?
                            );
                            Err("Filtering out this err!".into())
                        },
                        Some(deposit_info) => {
                            info!("✔ Deposit info from list: {:?}", deposit_info);
                            BtcOnEthEthTxInfo::new(
                                convert_satoshis_to_wei(tx_out.value),
                                deposit_info.address.clone(),
                                tx.txid(),
                                address.ok_or(NoneError("Could not unwrap BTC address!"))?,
                                if deposit_info.user_data.is_empty() {
                                    None
                                } else {
                                    Some(deposit_info.user_data.clone())
                                },
                                eth_token_address,
                            )
                        },
                    }
                })
                .filter(|maybe_eth_tx_infos| maybe_eth_tx_infos.is_ok())
                .collect::<Result<Vec<BtcOnEthEthTxInfo>>>()?,
        ))
    }

    pub fn from_btc_txs(
        txs: &[BtcTransaction],
        deposit_info: &DepositInfoHashMap,
        network: BtcNetwork,
        eth_token_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Parsing eth tx infos from `P2SH` transactions...");
        Ok(Self::new(
            txs.iter()
                .flat_map(|tx| Self::from_btc_tx(tx, deposit_info, network, eth_token_address))
                .flat_map(|eth_tx_infos| eth_tx_infos.0)
                .collect::<Vec<BtcOnEthEthTxInfo>>(),
        ))
    }
}

pub fn parse_eth_tx_infos_from_p2sh_deposits_and_add_to_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    info!("✔ Parsing eth tx infos from `P2SH` deposit txs in state...");
    BtcOnEthEthTxInfos::from_btc_txs(
        state.get_p2sh_deposit_txs()?,
        state.get_deposit_info_hash_map()?,
        state.btc_db_utils.get_btc_network_from_db()?,
        &EthDbUtils::new(state.db).get_btc_on_eth_smart_contract_address_from_db()?,
    )
    .and_then(|params| params.to_bytes())
    .map(|bytes| state.add_tx_infos(bytes))
}

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use std::str::FromStr;

    use common_btc::{
        convert_bytes_to_btc_pub_key_slice,
        create_hash_map_from_deposit_info_list,
        filter_p2sh_deposit_txs,
        test_utils::get_sample_btc_pub_key_slice,
    };
    use ethereum_types::H160 as EthAddress;

    use super::*;
    use crate::{
        bitcoin_crate_alias::{util::address::Address as BtcAddress, Txid},
        test_utils::get_sample_btc_block_n,
    };

    #[test]
    fn should_parse_eth_tx_infos_struct_from_p2sh_deposit_tx() {
        let pub_key = get_sample_btc_pub_key_slice();
        let expected_amount = convert_satoshis_to_wei(10000);
        let expected_num_results = 1;
        let expected_eth_address_bytes = hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap();
        let expected_btc_address = "2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2";
        let expected_tx_hash = "4d19fed40e7d1944c8590a8a2e21d1f16f65c060244277a3d207770d1c848352";
        let btc_network = BtcNetwork::Testnet;
        let block_and_id = get_sample_btc_block_n(1);
        let deposit_address_list = block_and_id.deposit_address_list.clone();
        let txs = block_and_id.block.txdata;
        let hash_map = create_hash_map_from_deposit_info_list(&deposit_address_list).unwrap();
        let tx = filter_p2sh_deposit_txs(&hash_map, &pub_key, &txs, btc_network).unwrap()[0].clone();
        let eth_token_address = EthAddress::default();
        let result = BtcOnEthEthTxInfos::from_btc_tx(&tx, &hash_map, btc_network, &eth_token_address).unwrap();
        assert_eq!(result[0].amount, expected_amount);
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result[0].originating_tx_hash.to_string(), expected_tx_hash);
        assert_eq!(result[0].originating_tx_address.to_string(), expected_btc_address);
        assert_eq!(
            result[0].destination_address.as_bytes(),
            &expected_eth_address_bytes[..]
        );
    }

    #[test]
    fn should_parse_eth_tx_infos_struct_from_p2sh_deposit_txs() {
        let expected_num_results = 1;
        let expected_amount = convert_satoshis_to_wei(10000);
        let expected_eth_address_bytes = hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap();
        let expected_btc_address = "2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2";
        let expected_tx_hash = "4d19fed40e7d1944c8590a8a2e21d1f16f65c060244277a3d207770d1c848352";
        let btc_network = BtcNetwork::Testnet;
        let block_and_id = get_sample_btc_block_n(1);
        let deposit_address_list = block_and_id.deposit_address_list.clone();
        let txs = block_and_id.block.txdata;
        let hash_map = create_hash_map_from_deposit_info_list(&deposit_address_list).unwrap();
        let eth_token_address = EthAddress::default();
        let result = BtcOnEthEthTxInfos::from_btc_txs(&txs, &hash_map, btc_network, &eth_token_address).unwrap();
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result[0].amount, expected_amount);
        assert_eq!(result[0].originating_tx_hash.to_string(), expected_tx_hash);
        assert_eq!(result[0].originating_tx_address.to_string(), expected_btc_address);
        assert_eq!(
            result[0].destination_address.as_bytes(),
            &expected_eth_address_bytes[..]
        );
    }

    #[test]
    fn should_parse_eth_tx_infos_struct_from_two_p2sh_deposit_txs() {
        let expected_num_results = 2;
        let expected_amount_1 = convert_satoshis_to_wei(314159);
        let expected_btc_address_1 = BtcAddress::from_str("2NCfNHvNAecRyXPBDaAkfgMLL7NjvPrC6GU").unwrap();
        let expected_amount_2 = convert_satoshis_to_wei(1000000);
        let expected_btc_address_2 = BtcAddress::from_str("2N6DgNSaX3D5rUYXuMM3b5Ujgw4sPrddSHp").unwrap();
        let expected_eth_address_1 =
            EthAddress::from_slice(&hex::decode("edb86cd455ef3ca43f0e227e00469c3bdfa40628").unwrap()[..]);
        let expected_eth_address_2 =
            EthAddress::from_slice(&hex::decode("7344d31d7025f72bd1d3c08645fa6b12d406fc05").unwrap()[..]);
        let expected_originating_tx_hash_1 =
            Txid::from_str("ee022f1be2981fbdd51f7c7ac2e07c1233bb7806e481df9c52b8077a628b2ea8").unwrap();
        let expected_originating_tx_hash_2 =
            Txid::from_str("130a150ff71f8cabf02d4315f7d61f801ced234c7fcc3144d858816033578110").unwrap();
        let pub_key_slice = convert_bytes_to_btc_pub_key_slice(
            &hex::decode("03a3bea6d8d15a38d9c96074d994c788bc1286d557ef5bdbb548741ddf265637ce").unwrap(),
        )
        .unwrap();
        let user_data = None;
        let eth_token_address = EthAddress::default();
        let expected_result_1 = BtcOnEthEthTxInfo::new(
            expected_amount_1,
            hex::encode(expected_eth_address_1),
            expected_originating_tx_hash_1,
            expected_btc_address_1,
            user_data.clone(),
            &eth_token_address,
        )
        .unwrap();
        let expected_result_2 = BtcOnEthEthTxInfo::new(
            expected_amount_2,
            hex::encode(expected_eth_address_2),
            expected_originating_tx_hash_2,
            expected_btc_address_2,
            user_data,
            &eth_token_address,
        )
        .unwrap();
        let btc_network = BtcNetwork::Testnet;
        let block_and_id = get_sample_btc_block_n(2);
        let deposit_address_list = block_and_id.deposit_address_list.clone();
        let txs = block_and_id.block.txdata;
        let hash_map = create_hash_map_from_deposit_info_list(&deposit_address_list).unwrap();
        let filtered_txs = filter_p2sh_deposit_txs(&hash_map, &pub_key_slice, &txs, btc_network).unwrap();
        let result =
            BtcOnEthEthTxInfos::from_btc_txs(&filtered_txs, &hash_map, btc_network, &eth_token_address).unwrap();
        let result_1 = result[0].clone();
        let result_2 = result[1].clone();
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result_1, expected_result_1);
        assert_eq!(result_2, expected_result_2);
    }
}
