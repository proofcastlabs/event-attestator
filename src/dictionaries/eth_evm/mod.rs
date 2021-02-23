use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::Address as EthAddress;

pub(crate) mod test_utils;

use crate::{
    chains::{eth::eth_state::EthState, evm::eth_state::EthState as EvmState},
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    dictionaries::dictionary_constants::ETH_EVM_DICTIONARY_KEY,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, DerefMut)]
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

    fn add(mut self, entry: EthEvmTokenDictionaryEntry) -> Self {
        info!("✔ Adding `EthEvmTokenDictionary` entry: {:?}...", entry);
        match self.contains(&entry) {
            true => {
                info!("Not adding new `EthEvmTokenDictionaryEntry` ∵ account name already extant!");
                self
            },
            false => {
                self.push(entry);
                self
            },
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

    pub fn add_and_update_in_db<D: DatabaseInterface>(self, entry: EthEvmTokenDictionaryEntry, db: &D) -> Result<Self> {
        let new_self = self.add(entry);
        new_self.save_to_db(db)?;
        Ok(new_self)
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
        match self.iter().find(|entry| &entry.eth_address == address) {
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

    pub fn is_eth_token_supported(&self, address: &EthAddress) -> bool {
        self.get_entry_via_eth_address(address).is_ok()
    }

    pub fn is_evm_token_supported(&self, address: &EthAddress) -> bool {
        self.get_entry_via_evm_address(address).is_ok()
    }

    pub fn to_eth_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|entry| entry.eth_address).collect()
    }

    pub fn to_evm_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|entry| entry.evm_address).collect()
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
}

impl EthEvmTokenDictionaryEntry {
    fn to_json(&self) -> EthEvmTokenDictionaryEntryJson {
        EthEvmTokenDictionaryEntryJson {
            evm_symbol: self.evm_symbol.to_string(),
            eth_symbol: self.eth_symbol.to_string(),
            evm_address: hex::encode(self.evm_address),
            eth_address: hex::encode(self.eth_address),
        }
    }

    pub fn from_json(json: &EthEvmTokenDictionaryEntryJson) -> Result<Self> {
        Ok(Self {
            evm_symbol: json.evm_symbol.clone(),
            eth_symbol: json.eth_symbol.clone(),
            eth_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.eth_address))?),
            evm_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.evm_address))?),
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
}

impl EthEvmTokenDictionaryEntryJson {
    pub fn from_str(json_string: &str) -> Result<Self> {
        match serde_json::from_str(json_string) {
            Ok(result) => Ok(result),
            Err(err) => Err(err.into()),
        }
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
