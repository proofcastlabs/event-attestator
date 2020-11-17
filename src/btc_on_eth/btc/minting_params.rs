use derive_more::{
    Deref,
    DerefMut,
    Constructor,
};
use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
    network::constants::Network as BtcNetwork,
    blockdata::transaction::Transaction as BtcTransaction,
};
use ethereum_types::{
    U256,
    Address as EthAddress,
};
use crate::{
    traits::DatabaseInterface,
    btc_on_eth::utils::convert_satoshis_to_ptoken,
    types::{
        Byte,
        Bytes,
        Result,
        NoneError,
    },
    chains::{
        eth::eth_utils::safely_convert_hex_to_eth_address,
        btc::{
            btc_state::BtcState,
            btc_constants::MINIMUM_REQUIRED_SATOSHIS,
            deposit_address_info::DepositInfoHashMap,
            btc_database_utils::get_btc_network_from_db,
        },
    },
};

pub fn parse_minting_params_from_p2sh_deposits_and_add_to_state<D>(
    state: BtcState<D>
) -> Result<BtcState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing minting params from `p2sh` deposit txs in state...");
    BtcOnEthMintingParams::from_btc_txs(
        state.get_p2sh_deposit_txs()?,
        state.get_deposit_info_hash_map()?,
        get_btc_network_from_db(&state.db)?,
    )
        .and_then(|params| state.add_btc_on_eth_minting_params(params))
}

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnEthMintingParams(pub Vec<BtcOnEthMintingParamStruct>);

impl BtcOnEthMintingParams {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn filter_out_value_too_low(&self) -> Result<BtcOnEthMintingParams> {
        info!("✔ Filtering out any minting params below a minimum of {} Satoshis...", MINIMUM_REQUIRED_SATOSHIS);
        let threshold = convert_satoshis_to_ptoken(MINIMUM_REQUIRED_SATOSHIS);
        Ok(BtcOnEthMintingParams::new(
            self
                .iter()
                .filter(|params| {
                    match params.amount >= threshold {
                        true => true,
                        false => {
                            info!("✘ Filtering minting params ∵ value too low: {:?}", params);
                            false
                        }
                    }
                })
                .cloned()
                .collect::<Vec<BtcOnEthMintingParamStruct>>()
        ))
    }

    fn from_btc_tx(tx: &BtcTransaction, deposit_info: &DepositInfoHashMap, network: BtcNetwork) -> Result<Self> {
        info!("✔ Parsing minting params from single `p2sh` transaction...");
        Ok(Self::new(
            tx
                .output
                .iter()
                .filter(|tx_out| tx_out.script_pubkey.is_p2sh())
                .map(|tx_out|
                    match BtcAddress::from_script(&tx_out.script_pubkey, network) {
                        None => {
                            info!("✘ Could not derive BTC address from tx: {:?}", tx);
                            (tx_out, None)
                        }
                        Some(address) => {
                            info!("✔ BTC address extracted from `tx_out`: {}", address);
                            (tx_out, Some(address))
                        }
                    }
                )
                .filter(|(_, maybe_address)| maybe_address.is_some())
                .map(|(tx_out, address)|
                    match deposit_info.get(
                        &address.clone().ok_or(NoneError("Could not unwrap BTC address!"))?
                    ) {
                        None => {
                            info!("✘ BTC address {} not in deposit list!", address
                                .ok_or(NoneError("Could not unwrap BTC address!"))?
                            );
                            Err("Filtering out this err!".into())
                        }
                        Some(deposit_info) => {
                            info!("✔ Deposit info from list: {:?}", deposit_info);
                            BtcOnEthMintingParamStruct::new(
                                convert_satoshis_to_ptoken(tx_out.value),
                                deposit_info.address.clone(),
                                tx.txid(),
                                address.ok_or(NoneError("Could not unwrap BTC address!"))?,
                            )
                        }
                    }
                 )
                .filter(|maybe_minting_params| maybe_minting_params.is_ok())
                .collect::<Result<Vec<BtcOnEthMintingParamStruct>>>()?
        ))
    }

    pub fn from_btc_txs(txs: &[BtcTransaction], deposit_info: &DepositInfoHashMap, network: BtcNetwork,) -> Result<Self> {
        info!("✔ Parsing minting params from `p2sh` transactions...");
        Ok(Self::new(
            txs
                .iter()
                .flat_map(|tx| Self::from_btc_tx(tx, deposit_info, network))
                .map(|minting_params| minting_params.0)
                .flatten()
                .collect::<Vec<BtcOnEthMintingParamStruct>>()
       ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcOnEthMintingParamStruct {
    pub amount: U256,
    pub eth_address: EthAddress,
    pub originating_tx_hash: sha256d::Hash,
    pub originating_tx_address: String,
}

impl BtcOnEthMintingParamStruct {
    pub fn new(
        amount: U256,
        eth_address_hex: String,
        originating_tx_hash: sha256d::Hash,
        originating_tx_address: BtcAddress,
    ) -> Result<BtcOnEthMintingParamStruct> {
        Ok(BtcOnEthMintingParamStruct {
            amount,
            originating_tx_hash,
            originating_tx_address: originating_tx_address.to_string(),
            eth_address: safely_convert_hex_to_eth_address(&eth_address_hex)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use ethereum_types::H160 as EthAddress;
    use bitcoin::{
        hashes::sha256d,
        util::address::Address as BtcAddress,
    };
    use crate::{
        chains::btc::{
            btc_test_utils::get_sample_minting_params,
            filter_p2sh_deposit_txs::filter_p2sh_deposit_txs,
            get_deposit_info_hash_map::create_hash_map_from_deposit_info_list,
            btc_test_utils::{
                get_sample_btc_block_n,
                get_sample_btc_pub_key_bytes,
            },
        },
    };

    #[test]
    fn should_filter_minting_params() {
        let expected_length_before = 3;
        let expected_length_after = 2;
        let minting_params = get_sample_minting_params();
        let threshold = convert_satoshis_to_ptoken(MINIMUM_REQUIRED_SATOSHIS);
        let length_before = minting_params.len();
        assert_eq!(length_before, expected_length_before);
        let result = minting_params.filter_out_value_too_low().unwrap();
        let length_after = result.len();
        assert_eq!(length_after, expected_length_after);
        result.iter().map(|params| assert!(params.amount >= threshold)).for_each(drop);
    }

    #[test]
    fn should_parse_minting_params_struct_from_p2sh_deposit_tx() {
        let pub_key = get_sample_btc_pub_key_bytes();
        let expected_amount = convert_satoshis_to_ptoken(10000);
        let expected_num_results = 1;
        let expected_eth_address_bytes = hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap();
        let expected_btc_address = "2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2";
        let expected_tx_hash = "4d19fed40e7d1944c8590a8a2e21d1f16f65c060244277a3d207770d1c848352";
        let btc_network = BtcNetwork::Testnet;
        let block_and_id = get_sample_btc_block_n(5).unwrap();
        let deposit_address_list = block_and_id.deposit_address_list.clone();
        let txs = block_and_id.block.txdata;
        let hash_map = create_hash_map_from_deposit_info_list(&deposit_address_list).unwrap();
        let tx = filter_p2sh_deposit_txs(&hash_map, &pub_key[..], &txs, btc_network).unwrap()[0].clone();
        let result = BtcOnEthMintingParams::from_btc_tx(&tx, &hash_map, btc_network).unwrap();
        assert_eq!(result[0].amount, expected_amount);
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result[0].originating_tx_hash.to_string(), expected_tx_hash);
        assert_eq!(result[0].originating_tx_address.to_string(), expected_btc_address);
        assert_eq!(result[0].eth_address.as_bytes(), &expected_eth_address_bytes[..]);
    }

    #[test]
    fn should_parse_minting_params_struct_from_p2sh_deposit_txs() {
        let expected_num_results = 1;
        let expected_amount = convert_satoshis_to_ptoken(10000);
        let expected_eth_address_bytes = hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap();
        let expected_btc_address = "2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2";
        let expected_tx_hash = "4d19fed40e7d1944c8590a8a2e21d1f16f65c060244277a3d207770d1c848352";
        let btc_network = BtcNetwork::Testnet;
        let block_and_id = get_sample_btc_block_n(5).unwrap();
        let deposit_address_list = block_and_id.deposit_address_list.clone();
        let txs = block_and_id.block.txdata;
        let hash_map = create_hash_map_from_deposit_info_list(&deposit_address_list).unwrap();
        let result = BtcOnEthMintingParams::from_btc_txs(&txs, &hash_map, btc_network).unwrap();
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result[0].amount, expected_amount);
        assert_eq!(result[0].originating_tx_hash.to_string(), expected_tx_hash);
        assert_eq!(result[0].originating_tx_address.to_string(), expected_btc_address);
        assert_eq!(result[0].eth_address.as_bytes(), &expected_eth_address_bytes[..]);
    }

    #[test]
    fn should_parse_minting_params_struct_from_two_p2sh_deposit_txs() {
        let expected_num_results = 2;
        let expected_amount_1 = convert_satoshis_to_ptoken(314159);
        let expected_btc_address_1 = BtcAddress::from_str("2NCfNHvNAecRyXPBDaAkfgMLL7NjvPrC6GU").unwrap();
        let expected_amount_2 = convert_satoshis_to_ptoken(1000000);
        let expected_btc_address_2 = BtcAddress::from_str("2N6DgNSaX3D5rUYXuMM3b5Ujgw4sPrddSHp").unwrap();
        let expected_eth_address_1 = EthAddress::from_slice(
            &hex::decode("edb86cd455ef3ca43f0e227e00469c3bdfa40628").unwrap()[..]
        );
        let expected_eth_address_2 = EthAddress::from_slice(
            &hex::decode("7344d31d7025f72bd1d3c08645fa6b12d406fc05").unwrap()[..]
        );
        let expected_originating_tx_hash_1 = sha256d::Hash::from_str(
            "ee022f1be2981fbdd51f7c7ac2e07c1233bb7806e481df9c52b8077a628b2ea8"
        ).unwrap();
        let expected_originating_tx_hash_2 = sha256d::Hash::from_str(
            "130a150ff71f8cabf02d4315f7d61f801ced234c7fcc3144d858816033578110"
        ).unwrap();
        let pub_key_bytes = hex::decode("03a3bea6d8d15a38d9c96074d994c788bc1286d557ef5bdbb548741ddf265637ce").unwrap();
        let expected_result_1 = BtcOnEthMintingParamStruct::new(
            expected_amount_1,
            hex::encode(expected_eth_address_1),
            expected_originating_tx_hash_1,
            expected_btc_address_1,
        ).unwrap();
        let expected_result_2 = BtcOnEthMintingParamStruct::new(
            expected_amount_2,
            hex::encode(expected_eth_address_2),
            expected_originating_tx_hash_2,
            expected_btc_address_2,
        ).unwrap();
        let btc_network = BtcNetwork::Testnet;
        let block_and_id = get_sample_btc_block_n(6).unwrap();
        let deposit_address_list = block_and_id.deposit_address_list.clone();
        let txs = block_and_id.block.txdata;
        let hash_map = create_hash_map_from_deposit_info_list(&deposit_address_list).unwrap();
        let filtered_txs = filter_p2sh_deposit_txs(&hash_map, &pub_key_bytes[..], &txs, btc_network,).unwrap();
        let result = BtcOnEthMintingParams::from_btc_txs(&filtered_txs, &hash_map, btc_network).unwrap();
        let result_1 = result[0].clone();
        let result_2 = result[1].clone();
        assert_eq!(result.len(), expected_num_results);
        assert_eq!(result_1, expected_result_1);
        assert_eq!(result_2, expected_result_2);
    }
}
