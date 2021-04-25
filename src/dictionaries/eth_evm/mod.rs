use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{
    chains::{eth::eth_state::EthState, evm::eth_state::EthState as EvmState},
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    dictionaries::dictionary_constants::ETH_EVM_DICTIONARY_KEY,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

pub(crate) mod test_utils;

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, DerefMut, Serialize, Deserialize)]
pub struct EthEvmTokenDictionary(pub Vec<EthEvmTokenDictionaryEntry>);

impl EthEvmTokenDictionary {
    pub fn to_json(&self) -> Result<EthEvmTokenDictionaryJson> {
        Ok(EthEvmTokenDictionaryJson::new(
            self.iter().map(|entry| entry.to_json()).collect(),
        ))
    }

    pub fn from_json(json: &EthEvmTokenDictionaryJson) -> Result<Self> {
        Ok(Self(
            json.iter()
                .map(|entry_json| EthEvmTokenDictionaryEntry::from_json(&entry_json))
                .collect::<Result<Vec<EthEvmTokenDictionaryEntry>>>()?,
        ))
    }

    fn to_bytes(&self) -> Result<Bytes> {
        self.to_json()?.to_bytes()
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        EthEvmTokenDictionaryJson::from_bytes(bytes).and_then(|json| Self::from_json(&json))
    }

    fn add(&mut self, entry: EthEvmTokenDictionaryEntry) {
        info!("✔ Adding `EthEvmTokenDictionary` entry: {:?}...", entry);
        if !self.contains(&entry) {
            self.push(entry);
        } else {
            info!("Not adding new `EthEvmTokenDictionaryEntry` ∵ account name already extant!");
        }
    }

    fn remove(mut self, entry: &EthEvmTokenDictionaryEntry) -> Self {
        info!("✔ Removing `EthEvmTokenDictionary` entry: {:?}...", entry);
        match self.contains(&entry) {
            false => self,
            true => {
                info!("Removing `EthEvmTokenDictionaryEntry`: {:?}", entry);
                self.retain(|x| x != entry);
                self
            },
        }
    }

    pub fn save_to_db<D: DatabaseInterface>(&self, db: &D) -> Result<()> {
        db.put(
            ETH_EVM_DICTIONARY_KEY.to_vec(),
            self.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_from_db<D: DatabaseInterface>(db: &D) -> Result<Self> {
        info!("✔ Getting `EthEvmTokenDictionaryJson` from db...");
        match db.get(ETH_EVM_DICTIONARY_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(_) => {
                info!("✔ No `EthEvmTokenDictionaryJson` in db! Initializing a new one...");
                Ok(Self::new(vec![]))
            },
        }
    }

    pub fn add_and_update_in_db<D: DatabaseInterface>(
        mut self,
        entry: EthEvmTokenDictionaryEntry,
        db: &D,
    ) -> Result<Self> {
        self.add(entry);
        self.save_to_db(db)?;
        Ok(self)
    }

    fn remove_and_update_in_db<D: DatabaseInterface>(self, entry: &EthEvmTokenDictionaryEntry, db: &D) -> Result<Self> {
        if self.contains(entry) {
            let new_self = self.remove(entry);
            new_self.save_to_db(db)?;
            return Ok(new_self);
        }
        Ok(self)
    }

    pub fn remove_entry_via_eth_address_and_update_in_db<D: DatabaseInterface>(
        self,
        eth_address: &EthAddress,
        db: &D,
    ) -> Result<Self> {
        self.get_entry_via_eth_address(eth_address)
            .and_then(|entry| self.remove_and_update_in_db(&entry, db))
    }

    pub fn get_entry_via_eth_address(&self, address: &EthAddress) -> Result<EthEvmTokenDictionaryEntry> {
        match self.iter().find(|ref entry| entry.eth_address == *address) {
            Some(entry) => Ok(entry.clone()),
            None => Err(format!("No `EthEvmTokenDictionaryEntry` exists with ETH address: {}", address).into()),
        }
    }

    pub fn get_entry_via_evm_address(&self, address: &EthAddress) -> Result<EthEvmTokenDictionaryEntry> {
        match self.iter().find(|entry| &entry.evm_address == address) {
            Some(entry) => Ok(entry.clone()),
            None => Err(format!("No `EthEvmTokenDictionaryEntry` exists with ETH address: {}", address).into()),
        }
    }

    pub fn get_evm_address_from_eth_address(&self, address: &EthAddress) -> Result<EthAddress> {
        self.get_entry_via_eth_address(address).map(|entry| entry.evm_address)
    }

    pub fn get_eth_address_from_evm_address(&self, address: &EthAddress) -> Result<EthAddress> {
        self.get_entry_via_evm_address(address).map(|entry| entry.eth_address)
    }

    pub fn is_evm_token_supported(&self, address: &EthAddress) -> bool {
        self.get_entry_via_evm_address(address).is_ok()
    }

    pub fn to_evm_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|entry| entry.evm_address).collect()
    }

    #[cfg(test)]
    pub fn from_str(s: &str) -> Result<Self> {
        let entry_jsons: Vec<EthEvmTokenDictionaryEntryJson> = serde_json::from_str(s)?;
        Ok(Self::new(
            entry_jsons
                .iter()
                .map(|ref entry_json| EthEvmTokenDictionaryEntry::from_json(entry_json))
                .collect::<Result<Vec<EthEvmTokenDictionaryEntry>>>()?,
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct EthEvmTokenDictionaryJson(pub Vec<EthEvmTokenDictionaryEntryJson>);

impl EthEvmTokenDictionaryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EthEvmTokenDictionaryEntry {
    pub eth_symbol: String,
    pub evm_symbol: String,
    pub evm_address: EthAddress,
    pub eth_address: EthAddress,
    pub eth_fee_basis_points: u64,
    pub evm_fee_basis_points: u64,
}

impl EthEvmTokenDictionaryEntry {
    fn to_json(&self) -> EthEvmTokenDictionaryEntryJson {
        EthEvmTokenDictionaryEntryJson {
            evm_symbol: self.evm_symbol.to_string(),
            eth_symbol: self.eth_symbol.to_string(),
            evm_address: hex::encode(self.evm_address),
            eth_address: hex::encode(self.eth_address),
            eth_fee_basis_points: Some(self.eth_fee_basis_points),
            evm_fee_basis_points: Some(self.evm_fee_basis_points),
        }
    }

    pub fn from_json(json: &EthEvmTokenDictionaryEntryJson) -> Result<Self> {
        Ok(Self {
            evm_symbol: json.evm_symbol.clone(),
            eth_symbol: json.eth_symbol.clone(),
            eth_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.eth_address))?),
            evm_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.evm_address))?),
            eth_fee_basis_points: json.eth_fee_basis_points.unwrap_or_default(),
            evm_fee_basis_points: json.evm_fee_basis_points.unwrap_or_default(),
        })
    }

    pub fn from_str(json_string: &str) -> Result<Self> {
        EthEvmTokenDictionaryEntryJson::from_str(json_string).and_then(|entry_json| Self::from_json(&entry_json))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EthEvmTokenDictionaryEntryJson {
    eth_symbol: String,
    evm_symbol: String,
    eth_address: String,
    evm_address: String,
    eth_fee_basis_points: Option<u64>,
    evm_fee_basis_points: Option<u64>,
}

impl EthEvmTokenDictionaryEntryJson {
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

pub fn get_eth_evm_token_dictionary_from_db_and_add_to_evm_state<D: DatabaseInterface>(
    state: EvmState<D>,
) -> Result<EvmState<D>> {
    info!("✔ Getting `EthEvmTokenDictionary` and adding to EVM state...");
    EthEvmTokenDictionary::get_from_db(&state.db).and_then(|dictionary| state.add_eth_evm_token_dictionary(dictionary))
}

pub fn get_eth_evm_token_dictionary_from_db_and_add_to_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Getting `EthEvmTokenDictionary` and adding to ETH state...");
    EthEvmTokenDictionary::get_from_db(&state.db).and_then(|dictionary| state.add_eth_evm_token_dictionary(dictionary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionaries::eth_evm::test_utils::{
        get_sample_eth_evm_dictionary,
        get_sample_eth_evm_dictionary_json_str,
    };

    #[test]
    fn should_get_dictionary_from_str() {
        let result = EthEvmTokenDictionary::from_str(&get_sample_eth_evm_dictionary_json_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn should_perform_dict_json_bytes_roundtrip() {
        let json = get_sample_eth_evm_dictionary().unwrap().to_json().unwrap();
        let bytes = json.to_bytes().unwrap();
        println!("{}", hex::encode(&bytes));
        let result = EthEvmTokenDictionaryJson::from_bytes(&bytes).unwrap();
        assert_eq!(result, json);
    }

    #[test]
    fn should_convert_bytes_to_dictionary() {
        // NOTE: This was the bytes encoding of a dictionary BEFORE extra optional args were added.
        // And so the test remains useful!
        let bytes = hex::decode("5b7b226574685f73796d626f6c223a22504e54222c2265766d5f73796d626f6c223a22504e54222c226574685f61646472657373223a2238396162333231353665343666343664303261646533666563626535666334323433623961616564222c2265766d5f61646472657373223a2264616163623061623666623334643234653861363762666131346266346439356434633761663932227d2c7b226574685f73796d626f6c223a224f5049554d222c2265766d5f73796d626f6c223a22704f5049554d222c226574685f61646472657373223a2238383838383838383838383963303063363736383930323964373835366161633130363565633131222c2265766d5f61646472657373223a2235363663656464323031663637653534326136383531613239353963316134343961303431393435227d2c7b226574685f73796d626f6c223a22505445524941222c2265766d5f73796d626f6c223a22505445524941222c226574685f61646472657373223a2230326563613931306362336137643433656263376538303238363532656435633662373032353962222c2265766d5f61646472657373223a2239663533373766613033646364343031366133333636396233383562653464306530326632376263227d2c7b226574685f73796d626f6c223a22424350222c2265766d5f73796d626f6c223a2270424350222c226574685f61646472657373223a2265346637323661646338653839633661363031376630316561646137373836356462323264613134222c2265766d5f61646472657373223a2261313134663839623439643661353834313662623037646265303935303263346633613139653266227d2c7b226574685f73796d626f6c223a22444546492b2b222c2265766d5f73796d626f6c223a2270444546492b2b222c226574685f61646472657373223a2238643163653336316562363865396530353537333434336334303764346133626564323362303333222c2265766d5f61646472657373223a2261653232653237643166373237623538353534396331306532363139326232626330313038326361227d2c7b226574685f73796d626f6c223a22434747222c2265766d5f73796d626f6c223a22434747222c226574685f61646472657373223a2231666532346632356231636636303962396334653765313264383032653336343064666135653433222c2265766d5f61646472657373223a2231363133393537313539653962306163366338306538323466376565613734386133326130616532227d5d").unwrap();
        let result = EthEvmTokenDictionaryJson::from_bytes(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn pnt_token_in_sample_dictionary_should_have_fees_set() {
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let pnt_address = EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap());
        let entry = dictionary.get_entry_via_eth_address(&pnt_address).unwrap();
        assert!(entry.eth_fee_basis_points > 0);
        assert!(entry.evm_fee_basis_points > 0);
    }
}
