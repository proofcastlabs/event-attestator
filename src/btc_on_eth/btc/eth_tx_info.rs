use bitcoin::{
    blockdata::transaction::Transaction as BtcTransaction,
    network::constants::Network as BtcNetwork,
    util::address::Address as BtcAddress,
    Txid,
};
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::{
    btc_on_eth::utils::{convert_satoshis_to_wei, convert_wei_to_satoshis},
    chains::{
        btc::{
            btc_constants::MINIMUM_REQUIRED_SATOSHIS,
            btc_metadata::ToMetadata,
            btc_state::BtcState,
            deposit_address_info::DepositInfoHashMap,
        },
        eth::{eth_database_utils::EthDbUtilsExt, eth_utils::safely_convert_hex_to_eth_address},
    },
    constants::FEE_BASIS_POINTS_DIVISOR,
    fees::fee_utils::sanity_check_basis_points_value,
    traits::DatabaseInterface,
    types::{Byte, Bytes, NoneError, Result},
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnEthEthTxInfos(pub Vec<BtcOnEthEthTxInfo>);

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcOnEthEthTxInfo {
    pub amount: U256,
    pub user_data: Option<Bytes>,
    pub originating_tx_hash: Txid,
    pub eth_token_address: EthAddress,
    pub originating_tx_address: String,
    pub destination_address: EthAddress,
}

pub fn parse_eth_tx_infos_from_p2sh_deposits_and_add_to_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    info!("✔ Parsing eth tx infos from `P2SH` deposit txs in state...");
    BtcOnEthEthTxInfos::from_btc_txs(
        state.get_p2sh_deposit_txs()?,
        state.get_deposit_info_hash_map()?,
        state.btc_db_utils.get_btc_network_from_db()?,
        &state.eth_db_utils.get_btc_on_eth_smart_contract_address_from_db()?,
    )
    .and_then(|params| state.add_btc_on_eth_eth_tx_infos(params))
}

impl BtcOnEthEthTxInfos {
    #[cfg(test)]
    pub fn sum(&self) -> U256 {
        self.iter().fold(U256::zero(), |a, params| a + params.amount)
    }

    pub fn calculate_fees(&self, basis_points: u64) -> Result<(Vec<u64>, u64)> {
        sanity_check_basis_points_value(basis_points).map(|_| {
            info!("✔ Calculating fees in `BtcOnEthEthTxInfos`...");
            let fees = self
                .iter()
                .map(|eth_tx_infos| eth_tx_infos.calculate_fee(basis_points))
                .collect::<Vec<u64>>();
            let total_fee = fees.iter().sum();
            info!("✔      Fees: {:?}", fees);
            info!("✔ Total fee: {:?}", fees);
            (fees, total_fee)
        })
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn filter_out_value_too_low(&self) -> Result<BtcOnEthEthTxInfos> {
        info!(
            "✔ Filtering out any eth tx infos below a minimum of {} Satoshis...",
            MINIMUM_REQUIRED_SATOSHIS
        );
        let threshold = convert_satoshis_to_wei(MINIMUM_REQUIRED_SATOSHIS);
        Ok(BtcOnEthEthTxInfos::new(
            self.iter()
                .filter(|params| match params.amount >= threshold {
                    true => true,
                    false => {
                        info!("✘ Filtering eth tx infos ∵ value too low: {:?}", params);
                        false
                    },
                })
                .cloned()
                .collect::<Vec<BtcOnEthEthTxInfo>>(),
        ))
    }

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
                    None => {
                        info!("✘ Could not derive BTC address from tx: {:?}", tx);
                        (tx_out, None)
                    },
                    Some(address) => {
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

impl BtcOnEthEthTxInfo {
    pub fn new(
        amount: U256,
        eth_address_hex: String,
        originating_tx_hash: Txid,
        originating_tx_address: BtcAddress,
        user_data: Option<Bytes>,
        eth_token_address: &EthAddress,
    ) -> Result<BtcOnEthEthTxInfo> {
        Ok(BtcOnEthEthTxInfo {
            amount,
            originating_tx_hash,
            originating_tx_address: originating_tx_address.to_string(),
            destination_address: safely_convert_hex_to_eth_address(&eth_address_hex)?,
            user_data,
            eth_token_address: *eth_token_address,
        })
    }

    fn to_satoshi_amount(&self) -> u64 {
        convert_wei_to_satoshis(self.amount)
    }

    pub fn calculate_fee(&self, basis_points: u64) -> u64 {
        (self.to_satoshi_amount() * basis_points) / FEE_BASIS_POINTS_DIVISOR
    }

    fn update_amount(&self, new_amount: U256) -> Self {
        let mut new_self = self.clone();
        new_self.amount = new_amount;
        new_self
    }

    pub fn subtract_satoshi_amount(&self, subtrahend: u64) -> Result<Self> {
        let self_amount_in_satoshis = self.to_satoshi_amount();
        if subtrahend > self_amount_in_satoshis {
            Err("Cannot subtract amount from `BtcOnEthEthTxInfo`: subtrahend too large!".into())
        } else {
            let amount_minus_fee = self_amount_in_satoshis - subtrahend;
            debug!(
                "Subtracted amount of {} from current eth tx infos amount of {} to get final amount of {}",
                subtrahend, self_amount_in_satoshis, amount_minus_fee
            );
            Ok(self.update_amount(convert_satoshis_to_wei(amount_minus_fee)))
        }
    }
}

impl ToMetadata for BtcOnEthEthTxInfo {
    fn get_user_data(&self) -> Option<Bytes> {
        self.user_data.clone()
    }

    fn get_originating_tx_address(&self) -> String {
        self.originating_tx_address.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{hashes::Hash, util::address::Address as BtcAddress};
    use ethereum_types::H160 as EthAddress;

    use super::*;
    use crate::{
        chains::{
            btc::{
                btc_chain_id::BtcChainId,
                btc_test_utils::{get_sample_btc_block_n, get_sample_btc_pub_key_slice, get_sample_eth_tx_infos},
                btc_utils::convert_bytes_to_btc_pub_key_slice,
                filter_p2sh_deposit_txs::filter_p2sh_deposit_txs,
                get_deposit_info_hash_map::create_hash_map_from_deposit_info_list,
            },
            eth::eth_constants::MAX_BYTES_FOR_ETH_USER_DATA,
        },
        errors::AppError,
        metadata::metadata_protocol_id::MetadataProtocolId,
    };

    #[test]
    fn should_filter_eth_tx_infos() {
        let expected_length_before = 3;
        let expected_length_after = 2;
        let eth_tx_infos = get_sample_eth_tx_infos();
        let threshold = convert_satoshis_to_wei(MINIMUM_REQUIRED_SATOSHIS);
        let length_before = eth_tx_infos.len();
        assert_eq!(length_before, expected_length_before);
        let result = eth_tx_infos.filter_out_value_too_low().unwrap();
        let length_after = result.len();
        assert_eq!(length_after, expected_length_after);
        result.iter().for_each(|params| assert!(params.amount >= threshold));
    }

    #[test]
    fn should_parse_eth_tx_infos_struct_from_p2sh_deposit_tx() {
        let pub_key = get_sample_btc_pub_key_slice();
        let expected_amount = convert_satoshis_to_wei(10000);
        let expected_num_results = 1;
        let expected_eth_address_bytes = hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap();
        let expected_btc_address = "2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2";
        let expected_tx_hash = "4d19fed40e7d1944c8590a8a2e21d1f16f65c060244277a3d207770d1c848352";
        let btc_network = BtcNetwork::Testnet;
        let block_and_id = get_sample_btc_block_n(5).unwrap();
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
        let block_and_id = get_sample_btc_block_n(5).unwrap();
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
        let block_and_id = get_sample_btc_block_n(6).unwrap();
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

    #[test]
    fn should_get_amount_in_satoshi() {
        let params = get_sample_eth_tx_infos()[0].clone();
        let result = params.to_satoshi_amount();
        let expected_result = 5000;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_satoshi_amount() {
        let params = get_sample_eth_tx_infos()[0].clone();
        let subtracted_params = params.subtract_satoshi_amount(1).unwrap();
        let expected_result = 4999;
        let result = subtracted_params.to_satoshi_amount();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_fee() {
        let params = get_sample_eth_tx_infos()[0].clone();
        let basis_points = 25;
        let expected_result = 12;
        let result = params.calculate_fee(basis_points);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_fees() {
        let basis_points = 25;
        let params = get_sample_eth_tx_infos();
        let (fees, total_fee) = params.calculate_fees(basis_points).unwrap();
        let expected_total_fee = 36;
        let expected_fees = vec![12, 12, 12];
        assert_eq!(total_fee, expected_total_fee);
        assert_eq!(fees, expected_fees);
    }

    #[test]
    fn should_error_subtracting_amount_if_subtrahend_is_too_large() {
        let params = get_sample_eth_tx_infos()[0].clone();
        let subtrahend = (params.amount + 1).as_u64();
        let expected_error = "Cannot subtract amount from `BtcOnEthEthTxInfo`: subtrahend too large!";
        match params.subtract_satoshi_amount(subtrahend) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_serde_eth_tx_infos() {
        let expected_serialization = "5b7b22616d6f756e74223a2230786332386632313963343030222c22757365725f64617461223a6e756c6c2c226f726967696e6174696e675f74785f68617368223a2239653864643239663038333938643761646639323532386163313133626363373336663761646364376339396565653034363861393932633831663365613938222c226574685f746f6b656e5f61646472657373223a22307830303030303030303030303030303030303030303030303030303030303030303030303030303030222c226f726967696e6174696e675f74785f61646472657373223a22324e324c48596274384b314b44426f6764365855473956427635594d36786566644d32222c2264657374696e6174696f6e5f61646472657373223a22307866656466653236313665623336363163623866656432373832663566306363393164353964636163227d5d";
        let amount = convert_satoshis_to_wei(1337);
        let originating_tx_address = BtcAddress::from_str("2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2").unwrap();
        let destination_address =
            EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap());
        let user_data = None;
        let eth_token_address = EthAddress::default();
        let originating_tx_hash =
            Txid::from_slice(&hex::decode("98eaf3812c998a46e0ee997ccdadf736c7bc13c18a5292df7a8d39089fd28d9e").unwrap())
                .unwrap();
        let eth_tx_info = BtcOnEthEthTxInfo::new(
            amount,
            hex::encode(destination_address),
            originating_tx_hash,
            originating_tx_address,
            user_data,
            &eth_token_address,
        )
        .unwrap();
        let eth_tx_infos = BtcOnEthEthTxInfos::new(vec![eth_tx_info]);
        let serialized_eth_tx_infos = eth_tx_infos.to_bytes().unwrap();
        assert_eq!(hex::encode(&serialized_eth_tx_infos), expected_serialization);
        let deserialized = BtcOnEthEthTxInfos::from_bytes(&serialized_eth_tx_infos).unwrap();
        assert_eq!(deserialized.len(), eth_tx_infos.len());
        deserialized
            .iter()
            .enumerate()
            .for_each(|(i, eth_tx_info)| assert_eq!(eth_tx_info, &eth_tx_infos[i]));
    }

    #[test]
    fn should_convert_eth_tx_infos_to_metadata_bytes() {
        let mut eth_tx_info = get_sample_eth_tx_infos()[0].clone();
        eth_tx_info.user_data = Some(hex::decode("d3caffc0ff33").unwrap());
        let expected_result = Some(hex::decode("0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001ec97de0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000006d3caffc0ff330000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002231333643544552616f636d38644c6245747a436146744a4a58396a6646686e43684b000000000000000000000000000000000000000000000000000000000000").unwrap());
        let btc_chain_id = BtcChainId::Bitcoin;
        let destination_protocol_id = MetadataProtocolId::Ethereum;
        let result = eth_tx_info
            .maybe_to_metadata_bytes(&btc_chain_id, MAX_BYTES_FOR_ETH_USER_DATA, &destination_protocol_id)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_convert_eth_tx_infos_to_metadata_bytes_if_user_data_too_large() {
        let mut eth_tx_info = get_sample_eth_tx_infos()[0].clone();
        eth_tx_info.user_data = Some(vec![0u8; MAX_BYTES_FOR_ETH_USER_DATA + 1]);
        let btc_chain_id = BtcChainId::Bitcoin;
        let destination_protocol_id = MetadataProtocolId::Ethereum;
        let result = eth_tx_info
            .maybe_to_metadata_bytes(&btc_chain_id, MAX_BYTES_FOR_ETH_USER_DATA, &destination_protocol_id)
            .unwrap();
        assert!(result.is_none());
    }
}
