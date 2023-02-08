use std::str::FromStr;

use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::{
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    dictionaries::{
        dictionary_constants::EVM_ALGO_DICTIONARY_KEY,
        evm_algo::dictionary_entry::{EvmAlgoTokenDictionaryEntry, EvmAlgoTokenDictionaryEntryJson},
    },
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};

#[derive(Debug, Clone, Eq, Default, PartialEq, Constructor, Deref, DerefMut, Serialize, Deserialize)]
pub struct EvmAlgoTokenDictionary(pub Vec<EvmAlgoTokenDictionaryEntry>);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct EvmAlgoTokenDictionaryJson(pub Vec<EvmAlgoTokenDictionaryEntryJson>);

impl EvmAlgoTokenDictionaryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

impl EvmAlgoTokenDictionary {
    pub fn convert_evm_amount_to_algo_amount(&self, address: &EthAddress, amount: U256) -> Result<U256> {
        self.get_entry_via_evm_address(address)
            .and_then(|entry| entry.convert_evm_amount_to_algo_amount(amount))
    }

    pub fn convert_algo_amount_to_evm_amount(&self, asset_id: u64, amount: u64) -> Result<U256> {
        self.get_entry_via_asset_id(asset_id)
            .and_then(|entry| entry.convert_algo_amount_to_evm_amount(amount))
    }

    pub fn get_entry_via_evm_address(&self, address: &EthAddress) -> Result<EvmAlgoTokenDictionaryEntry> {
        match self.iter().find(|entry| &entry.evm_address == address) {
            Some(entry) => Ok(entry.clone()),
            None => Err(format!("No `EvmAlgoTokenDictionaryEntry` exists with EVM address: {}", address).into()),
        }
    }

    pub fn get_entry_via_asset_id(&self, asset_id: u64) -> Result<EvmAlgoTokenDictionaryEntry> {
        match self.iter().find(|entry| entry.algo_asset_id == asset_id) {
            Some(entry) => Ok(entry.clone()),
            None => Err(format!(
                "No `EvmAlgoTokenDictionaryEntry` exists with ALGO asset ID: {}",
                asset_id
            )
            .into()),
        }
    }

    pub fn to_json(&self) -> Result<EvmAlgoTokenDictionaryJson> {
        Ok(EvmAlgoTokenDictionaryJson::new(
            self.iter()
                .map(|entry| entry.to_json())
                .collect::<Result<Vec<EvmAlgoTokenDictionaryEntryJson>>>()?,
        ))
    }

    pub fn from_json(json: &EvmAlgoTokenDictionaryJson) -> Result<Self> {
        Ok(Self(
            json.iter()
                .map(EvmAlgoTokenDictionaryEntry::from_json)
                .collect::<Result<Vec<EvmAlgoTokenDictionaryEntry>>>()?,
        ))
    }

    fn to_bytes(&self) -> Result<Bytes> {
        self.to_json()?.to_bytes()
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        EvmAlgoTokenDictionaryJson::from_bytes(bytes).and_then(|json| Self::from_json(&json))
    }

    fn add(&self, entry: EvmAlgoTokenDictionaryEntry) -> Self {
        let mut new_self = self.clone();
        if self.contains(&entry) {
            info!("✘ Not adding new `EvmAlgoTokenDictionaryEntry` ∵ entry already extant!");
            new_self
        } else {
            info!("✔ Adding `EvmAlgoTokenDictionary` entry: {:?}...", entry);
            new_self.push(entry);
            new_self
        }
    }

    fn remove(&self, entry: &EvmAlgoTokenDictionaryEntry) -> Self {
        let mut new_self = self.clone();
        match self.contains(entry) {
            false => {
                info!(
                    "✔ Not removing `EvmAlgoTokenDictionary` entry ∵ it's not in the dictionary! {:?}",
                    entry
                );
                new_self
            },
            true => {
                info!("✔ Removing `EvmAlgoTokenDictionaryEntry`: {:?}", entry);
                new_self.retain(|x| x != entry);
                new_self
            },
        }
    }

    fn save_in_db<D: DatabaseInterface>(&self, db: &D) -> Result<()> {
        db.put(
            EVM_ALGO_DICTIONARY_KEY.to_vec(),
            self.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_from_db<D: DatabaseInterface>(db: &D) -> Result<Self> {
        info!("✔ Getting `EvmAlgoTokenDictionaryJson` from db...");
        match db.get(EVM_ALGO_DICTIONARY_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(_) => {
                info!("✘ No `EvmAlgoTokenDictionaryJson` in db! Initializing a new one...");
                Ok(Self::new(vec![]))
            },
        }
    }

    pub fn add_and_update_in_db<D: DatabaseInterface>(&self, entry: EvmAlgoTokenDictionaryEntry, db: &D) -> Result<()> {
        self.add(entry).save_in_db(db)
    }

    fn remove_and_update_in_db<D: DatabaseInterface>(&self, entry: &EvmAlgoTokenDictionaryEntry, db: &D) -> Result<()> {
        if self.contains(entry) {
            info!("✔ Removing entry & updating in db...");
            self.remove(entry).save_in_db(db)
        } else {
            info!("✘ Not removing entry || updating in db ∵ entry not extant!");
            Ok(())
        }
    }

    pub fn remove_entry_via_evm_address_and_update_in_db<D: DatabaseInterface>(
        &self,
        eth_address: &EthAddress,
        db: &D,
    ) -> Result<()> {
        self.get_entry_via_evm_address(eth_address)
            .and_then(|entry| self.remove_and_update_in_db(&entry, db))
    }

    pub fn is_evm_token_supported(&self, address: &EthAddress) -> bool {
        self.get_entry_via_evm_address(address).is_ok()
    }

    pub fn is_algo_asset_supported(&self, asset_id: u64) -> bool {
        self.get_entry_via_asset_id(asset_id).is_ok()
    }

    pub fn to_evm_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|entry| entry.evm_address).collect()
    }

    pub fn get_algo_asset_id_from_evm_address(&self, evm_address: &EthAddress) -> Result<u64> {
        self.get_entry_via_evm_address(evm_address)
            .map(|entry| entry.algo_asset_id)
    }

    pub fn get_evm_address_from_asset_id(&self, asset_id: u64) -> Result<EthAddress> {
        self.get_entry_via_asset_id(asset_id).map(|entry| entry.evm_address)
    }

    pub fn to_algo_asset_ids(&self) -> Vec<u64> {
        self.iter().map(|entry| entry.algo_asset_id).collect()
    }
}

impl FromStr for EvmAlgoTokenDictionary {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_json(&serde_json::from_str::<EvmAlgoTokenDictionaryJson>(s)?)
    }
}

impl FromStr for EvmAlgoTokenDictionaryJson {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::new(serde_json::from_str::<Vec<EvmAlgoTokenDictionaryEntryJson>>(
            s,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{test_utils::get_test_database, utils::convert_hex_to_eth_address};

    fn get_sample_entry_1() -> EvmAlgoTokenDictionaryEntry {
        EvmAlgoTokenDictionaryEntry {
            algo_asset_id: 42,
            evm_address: convert_hex_to_eth_address("0x0c6f292ddd1997e7712fC03b32F7e97503f689e9").unwrap(),
            algo_decimals: 8,
            evm_decimals: 18,
            algo_symbol: "ALGO".to_string(),
            evm_symbol: "EVM".to_string(),
        }
    }

    fn get_sample_entry_2() -> EvmAlgoTokenDictionaryEntry {
        EvmAlgoTokenDictionaryEntry {
            algo_asset_id: 666,
            evm_address: convert_hex_to_eth_address("0x5832e106799962e23d1d5b512cdb01ee76ef6f4d").unwrap(),
            algo_decimals: 10,
            evm_decimals: 10,
            algo_symbol: "ALGO".to_string(),
            evm_symbol: "EVM".to_string(),
        }
    }

    fn get_sample_dictionary() -> EvmAlgoTokenDictionary {
        EvmAlgoTokenDictionary::new(vec![get_sample_entry_1(), get_sample_entry_2()])
    }

    #[test]
    fn should_get_entry_via_evm_address() {
        let dict = get_sample_dictionary();
        let evm_address = convert_hex_to_eth_address("0x5832e106799962e23d1d5b512cdb01ee76ef6f4d").unwrap();
        let result = dict.get_entry_via_evm_address(&evm_address).unwrap();
        let expected_result = get_sample_entry_2();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_entry_via_algo_address() {
        let dict = get_sample_dictionary();
        let algo_asset_id = 42;
        let result = dict.get_entry_via_asset_id(algo_asset_id).unwrap();
        let expected_result = get_sample_entry_1();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serde_evm_algo_dictionary_to_and_from_json() {
        let dict = get_sample_dictionary();
        let json = dict.to_json().unwrap();
        let result = EvmAlgoTokenDictionary::from_json(&json).unwrap();
        assert_eq!(result, dict);
    }

    #[test]
    fn should_serde_evm_algo_dictionary_to_and_from_bytes() {
        let dict = get_sample_dictionary();
        let bytes = dict.to_bytes().unwrap();
        let result = EvmAlgoTokenDictionary::from_bytes(&bytes).unwrap();
        assert_eq!(result, dict);
    }

    #[test]
    fn should_add_entry() {
        let dict = EvmAlgoTokenDictionary::new(vec![get_sample_entry_1()]);
        let entry_2 = get_sample_entry_2();
        let address = convert_hex_to_eth_address("0x5832e106799962e23d1d5b512cdb01ee76ef6f4d").unwrap();
        assert!(dict.get_entry_via_evm_address(&address).is_err());
        let updated_dict = dict.add(entry_2.clone());
        let result = updated_dict.get_entry_via_evm_address(&address).unwrap();
        assert_eq!(result, entry_2);
    }

    #[test]
    fn should_remove_entry() {
        let dict = get_sample_dictionary();
        let address = convert_hex_to_eth_address("0x5832e106799962e23d1d5b512cdb01ee76ef6f4d").unwrap();
        let entry_2 = get_sample_entry_2();
        assert_eq!(dict.get_entry_via_evm_address(&address).unwrap(), entry_2);
        let updated_dict = dict.remove(&entry_2);
        assert!(updated_dict.get_entry_via_evm_address(&address).is_err());
    }

    #[test]
    fn should_initialize_new_dictionary_if_none_in_db() {
        let db = get_test_database();
        let result = EvmAlgoTokenDictionary::get_from_db(&db).unwrap();
        let expected_result = EvmAlgoTokenDictionary::new(vec![]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_save_and_get_from_db() {
        let db = get_test_database();
        let dict = get_sample_dictionary();
        dict.save_in_db(&db).unwrap();
        let result = EvmAlgoTokenDictionary::get_from_db(&db).unwrap();
        assert_eq!(result, dict);
    }

    #[test]
    fn evm_address_should_be_supported() {
        let dict = get_sample_dictionary();
        let address = convert_hex_to_eth_address("0x5832e106799962e23d1d5b512cdb01ee76ef6f4d").unwrap();
        let result = dict.is_evm_token_supported(&address);
        assert!(result);
    }

    #[test]
    fn algo_asset_id_should_be_supported() {
        let dict = get_sample_dictionary();
        let asset_id = 42;
        let result = dict.is_algo_asset_supported(asset_id);
        assert!(result);
    }

    #[test]
    fn should_get_evm_addresses() {
        let dict = get_sample_dictionary();
        let result = dict.to_evm_addresses();
        let expected_result = vec![
            convert_hex_to_eth_address("0x0c6f292ddd1997e7712fC03b32F7e97503f689e9").unwrap(),
            convert_hex_to_eth_address("0x5832e106799962e23d1d5b512cdb01ee76ef6f4d").unwrap(),
        ];
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_asset_ids() {
        let dict = get_sample_dictionary();
        let result = dict.to_algo_asset_ids();
        let expected_result = vec![42, 666];
        assert_eq!(result, expected_result);
    }

    fn get_sample_dictionary_string() -> String {
        json!([
            {
                "evm_decimals": 1,
                "algo_decimals": 1,
                "evm_address": "0x1a86F100DFc0d572E3D3fe4742075c768C442319",
                "algo_asset_id": 666,
                "evm_symbol": "EVM",
                "algo_symbol": "ALGO",

            },
            {
                "evm_decimals": 2,
                "algo_decimals": 2,
                "evm_address": "0x0D0707963952f2fBA59dD06f2b425ace40b492Fe",
                "algo_asset_id": 1337,
                "evm_symbol": "EVM",
                "algo_symbol": "ALGO",

            }
        ])
        .to_string()
    }

    #[test]
    fn should_get_evm_algo_dictionary_from_str() {
        let s = get_sample_dictionary_string();
        let result = EvmAlgoTokenDictionary::from_str(&s);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_asset_id_from_evm_address() {
        let dict = get_sample_dictionary();
        let evm_address = convert_hex_to_eth_address("0x5832e106799962e23d1d5b512cdb01ee76ef6f4d").unwrap();
        let result = dict.get_algo_asset_id_from_evm_address(&evm_address).unwrap();
        let expected_result = 666;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_evm_address_from_asset_id() {
        let dict = get_sample_dictionary();
        let asset_id = 666;
        let result = dict.get_evm_address_from_asset_id(asset_id).unwrap();
        let expected_result = convert_hex_to_eth_address("0x5832e106799962e23d1d5b512cdb01ee76ef6f4d").unwrap();
        assert_eq!(result, expected_result);
    }
}
