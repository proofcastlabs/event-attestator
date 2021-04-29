use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::{
    chains::{eth::eth_state::EthState, evm::eth_state::EthState as EvmState},
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    dictionaries::dictionary_constants::ETH_EVM_DICTIONARY_KEY,
    fees::fee_utils::get_last_withdrawal_date_as_human_readable_string,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::{get_unix_timestamp, strip_hex_prefix},
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

    fn save_in_db<D: DatabaseInterface>(&self, db: &D) -> Result<()> {
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
        self.save_in_db(db)?;
        Ok(self)
    }

    fn remove_and_update_in_db<D: DatabaseInterface>(self, entry: &EthEvmTokenDictionaryEntry, db: &D) -> Result<Self> {
        if self.contains(entry) {
            let new_self = self.remove(entry);
            new_self.save_in_db(db)?;
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

    fn get_eth_fee_basis_points(&self, eth_address: &EthAddress) -> Result<u64> {
        Ok(self.get_entry_via_eth_address(eth_address)?.eth_fee_basis_points)
    }

    fn get_evm_fee_basis_points(&self, evm_address: &EthAddress) -> Result<u64> {
        Ok(self.get_entry_via_evm_address(evm_address)?.evm_fee_basis_points)
    }

    pub fn get_fee_basis_points(&self, address: &EthAddress) -> Result<u64> {
        self.get_eth_fee_basis_points(address)
            .or_else(|_| self.get_evm_fee_basis_points(address))
    }

    fn get_entry_via_address(&self, address: &EthAddress) -> Result<EthEvmTokenDictionaryEntry> {
        self.get_entry_via_eth_address(address)
            .or_else(|_| self.get_entry_via_evm_address(address))
    }

    pub fn replace_entry(
        &mut self,
        entry_to_remove: &EthEvmTokenDictionaryEntry,
        entry_to_add: EthEvmTokenDictionaryEntry,
    ) -> Self {
        self.add(entry_to_add);
        self.clone().remove(entry_to_remove)
    }

    pub fn increment_accrued_fee(&mut self, address: &EthAddress, addend: U256) -> Result<Self> {
        self.get_entry_via_address(address)
            .map(|entry| self.replace_entry(&entry, entry.add_to_accrued_fees(addend)))
    }

    pub fn increment_accrued_fees(&mut self, fee_tuples: Vec<(EthAddress, U256)>) -> Result<Self> {
        fee_tuples.iter().try_fold(self.clone(), |mut s, (address, addend)| {
            s.increment_accrued_fee(address, *addend)
        })
    }

    pub fn increment_accrued_fees_and_save_in_db<D: DatabaseInterface>(
        &mut self,
        db: &D,
        fee_tuples: Vec<(EthAddress, U256)>,
    ) -> Result<Self> {
        self.increment_accrued_fees(fee_tuples).and_then(|new_dictionary| {
            new_dictionary.save_in_db(db)?;
            Ok(new_dictionary)
        })
    }

    fn change_eth_fee_basis_points(&mut self, eth_address: &EthAddress, new_fee: u64) -> Result<Self> {
        info!(
            "Changing ETH fee basis points for address {} to {}...",
            eth_address, new_fee
        );
        self.get_entry_via_eth_address(eth_address)
            .map(|entry| self.replace_entry(&entry, entry.change_eth_fee_basis_points(new_fee)))
    }

    fn change_evm_fee_basis_points(&mut self, evm_address: &EthAddress, new_fee: u64) -> Result<Self> {
        info!(
            "Changing EVM fee basis points for address {} to {}...",
            evm_address, new_fee
        );
        self.get_entry_via_evm_address(evm_address)
            .map(|entry| self.replace_entry(&entry, entry.change_evm_fee_basis_points(new_fee)))
    }

    fn change_fee_basis_points(&mut self, address: &EthAddress, new_fee: u64) -> Result<Self> {
        self.change_eth_fee_basis_points(address, new_fee)
            .or_else(|_| self.change_evm_fee_basis_points(address, new_fee))
    }

    pub fn change_fee_basis_points_and_update_in_db<D: DatabaseInterface>(
        &mut self,
        db: &D,
        address: &EthAddress,
        new_fee: u64,
    ) -> Result<()> {
        self.change_fee_basis_points(address, new_fee)
            .and_then(|updated_dictionary| updated_dictionary.save_in_db(db))
    }

    fn set_last_withdrawal_timestamp_in_entry(&mut self, address: &EthAddress, timestamp: u64) -> Result<Self> {
        self.get_entry_via_address(address)
            .map(|entry| self.replace_entry(&entry, entry.set_last_withdrawal_timestamp(timestamp)))
    }

    fn zero_accrued_fees_in_entry(&mut self, address: &EthAddress) -> Result<Self> {
        self.get_entry_via_address(address)
            .map(|entry| self.replace_entry(&entry, entry.zero_accrued_fees()))
    }

    fn get_fee_withdrawal_amount(&self, address: &EthAddress) -> Result<U256> {
        self.get_entry_via_address(address).map(|entry| entry.accrued_fees)
    }

    pub fn withdraw_fees<D: DatabaseInterface>(&mut self, db: &D, address: &EthAddress) -> Result<(EthAddress, U256)> {
        let token_address = self.get_entry_via_address(address)?.eth_address;
        let withdrawal_amount = self.get_fee_withdrawal_amount(address)?;
        self.set_last_withdrawal_timestamp_in_entry(address, get_unix_timestamp()?)
            .and_then(|mut dict| dict.zero_accrued_fees_in_entry(address))
            .and_then(|dict| dict.save_in_db(db))
            .map(|_| (token_address, withdrawal_amount))
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
    pub accrued_fees: U256,
    pub last_withdrawal: u64,
    pub accrued_fees_human_readable: u128,
    pub last_withdrawal_human_readable: String,
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
            accrued_fees: Some(self.accrued_fees.as_u128()),
            last_withdrawal: Some(self.last_withdrawal),
        }
    }

    pub fn from_json(json: &EthEvmTokenDictionaryEntryJson) -> Result<Self> {
        let timestamp = json.last_withdrawal.unwrap_or_default();
        let accrued_fees = U256::from(json.accrued_fees.unwrap_or_default());
        Ok(Self {
            evm_symbol: json.evm_symbol.clone(),
            eth_symbol: json.eth_symbol.clone(),
            eth_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.eth_address))?),
            evm_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.evm_address))?),
            eth_fee_basis_points: json.eth_fee_basis_points.unwrap_or_default(),
            evm_fee_basis_points: json.evm_fee_basis_points.unwrap_or_default(),
            accrued_fees_human_readable: accrued_fees.as_u128(),
            last_withdrawal: timestamp,
            last_withdrawal_human_readable: get_last_withdrawal_date_as_human_readable_string(timestamp),
            accrued_fees,
        })
    }

    pub fn from_str(json_string: &str) -> Result<Self> {
        EthEvmTokenDictionaryEntryJson::from_str(json_string).and_then(|entry_json| Self::from_json(&entry_json))
    }

    pub fn add_to_accrued_fees(&self, addend: U256) -> Self {
        let new_accrued_fees = self.accrued_fees + addend;
        debug!("Adding to accrued fees in {:?}...", self);
        debug!(
            "Updating accrued fees from {} to {}...",
            self.accrued_fees, new_accrued_fees
        );
        Self {
            eth_symbol: self.eth_symbol.clone(),
            evm_symbol: self.evm_symbol.clone(),
            evm_address: self.evm_address,
            eth_address: self.eth_address,
            eth_fee_basis_points: self.eth_fee_basis_points,
            evm_fee_basis_points: self.evm_fee_basis_points,
            accrued_fees: new_accrued_fees,
            accrued_fees_human_readable: new_accrued_fees.as_u128(),
            last_withdrawal: self.last_withdrawal,
            last_withdrawal_human_readable: self.last_withdrawal_human_readable.clone(),
        }
    }

    pub fn change_eth_fee_basis_points(&self, new_fee: u64) -> Self {
        debug!(
            "Changing ETH fee basis points for address {} from {} to {}...",
            self.eth_address, self.eth_fee_basis_points, new_fee
        );
        Self {
            eth_symbol: self.eth_symbol.clone(),
            evm_symbol: self.evm_symbol.clone(),
            evm_address: self.evm_address,
            eth_address: self.eth_address,
            eth_fee_basis_points: new_fee,
            evm_fee_basis_points: self.evm_fee_basis_points,
            accrued_fees: self.accrued_fees,
            accrued_fees_human_readable: self.accrued_fees_human_readable,
            last_withdrawal: self.last_withdrawal,
            last_withdrawal_human_readable: self.last_withdrawal_human_readable.clone(),
        }
    }

    pub fn change_evm_fee_basis_points(&self, new_fee: u64) -> Self {
        debug!(
            "Changing EVM fee basis points for address {} from {} to {}...",
            self.evm_address, self.evm_fee_basis_points, new_fee
        );
        Self {
            eth_symbol: self.eth_symbol.clone(),
            evm_symbol: self.evm_symbol.clone(),
            evm_address: self.evm_address,
            eth_address: self.eth_address,
            eth_fee_basis_points: self.eth_fee_basis_points,
            evm_fee_basis_points: new_fee,
            accrued_fees: self.accrued_fees,
            accrued_fees_human_readable: self.accrued_fees_human_readable,
            last_withdrawal: self.last_withdrawal,
            last_withdrawal_human_readable: self.last_withdrawal_human_readable.clone(),
        }
    }

    fn set_last_withdrawal_timestamp(&self, timestamp: u64) -> Self {
        let timestamp_human_readable = get_last_withdrawal_date_as_human_readable_string(timestamp);
        debug!("Setting withdrawal date to {}", timestamp_human_readable);
        Self {
            eth_symbol: self.eth_symbol.clone(),
            evm_symbol: self.evm_symbol.clone(),
            evm_address: self.evm_address,
            eth_address: self.eth_address,
            eth_fee_basis_points: self.eth_fee_basis_points,
            evm_fee_basis_points: self.evm_fee_basis_points,
            accrued_fees: self.accrued_fees,
            accrued_fees_human_readable: self.accrued_fees_human_readable,
            last_withdrawal: timestamp,
            last_withdrawal_human_readable: timestamp_human_readable,
        }
    }

    fn zero_accrued_fees(&self) -> Self {
        debug!("Zeroing accrued fees in {:?}...", self);
        Self {
            eth_symbol: self.eth_symbol.clone(),
            evm_symbol: self.evm_symbol.clone(),
            evm_address: self.evm_address,
            eth_address: self.eth_address,
            eth_fee_basis_points: self.eth_fee_basis_points,
            evm_fee_basis_points: self.evm_fee_basis_points,
            accrued_fees: U256::zero(),
            accrued_fees_human_readable: 0,
            last_withdrawal: self.last_withdrawal,
            last_withdrawal_human_readable: self.last_withdrawal_human_readable.clone(),
        }
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
    accrued_fees: Option<u128>,
    last_withdrawal: Option<u64>,
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
    use crate::{
        dictionaries::eth_evm::test_utils::{get_sample_eth_evm_dictionary, get_sample_eth_evm_dictionary_json_str},
        test_utils::get_test_database,
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

    #[test]
    fn should_get_eth_fee_basis_points() {
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let eth_address = EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap());
        let result = dictionary.get_eth_fee_basis_points(&eth_address).unwrap();
        let expected_result = 10;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_evm_fee_basis_points() {
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let evm_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let result = dictionary.get_evm_fee_basis_points(&evm_address).unwrap();
        let expected_result = 20;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_fee_basis_points() {
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let evm_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let result = dictionary.get_fee_basis_points(&evm_address).unwrap();
        let expected_result = 20;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_add_to_accrued_fees_in_dictionary_entry() {
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let evm_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let entry = dictionary.get_entry_via_evm_address(&evm_address).unwrap();
        assert_eq!(entry.last_withdrawal, 0);
        assert_eq!(entry.accrued_fees, U256::zero());
        let fee_to_add = U256::from(1337);
        let result = entry.add_to_accrued_fees(fee_to_add);
        assert_eq!(result.accrued_fees, fee_to_add);
    }

    #[test]
    fn should_get_entry_via_address() {
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let evm_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let result = dictionary.get_entry_via_address(&evm_address).unwrap();
        assert_eq!(result.evm_address, evm_address);
    }

    #[test]
    fn should_increment_accrued_fees() {
        let mut dictionary = get_sample_eth_evm_dictionary().unwrap();
        let fee_1 = U256::from(666);
        let fee_2 = U256::from(1337);
        let address_1 = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let address_2 = EthAddress::from_slice(&hex::decode("888888888889c00c67689029d7856aac1065ec11").unwrap());
        let fee_tuples = vec![(address_1, fee_1), (address_2, fee_2)];
        let entry_1_before = dictionary.get_entry_via_address(&address_1).unwrap();
        let entry_2_before = dictionary.get_entry_via_address(&address_2).unwrap();
        assert_eq!(entry_1_before.accrued_fees, U256::zero());
        assert_eq!(entry_2_before.accrued_fees, U256::zero());
        assert_eq!(entry_1_before.last_withdrawal, 0);
        assert_eq!(entry_2_before.last_withdrawal, 0);
        let result = dictionary.increment_accrued_fees(fee_tuples).unwrap();
        let entry_1_after = result.get_entry_via_address(&address_1).unwrap();
        let entry_2_after = result.get_entry_via_address(&address_2).unwrap();
        assert_eq!(entry_1_after.accrued_fees, fee_1);
        assert_eq!(entry_2_after.accrued_fees, fee_2);
    }

    #[test]
    fn should_change_eth_fee_basis_points() {
        let new_fee = 1337;
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let eth_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let entry = dictionary.get_entry_via_address(&eth_address).unwrap();
        let fee_before = entry.eth_fee_basis_points;
        assert_ne!(fee_before, new_fee);
        let result = entry.change_eth_fee_basis_points(new_fee);
        assert_eq!(result.eth_fee_basis_points, new_fee);
    }

    #[test]
    fn should_change_evm_fee_basis_points() {
        let new_fee = 1337;
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let evm_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let entry = dictionary.get_entry_via_address(&evm_address).unwrap();
        let fee_before = entry.evm_fee_basis_points;
        assert_ne!(fee_before, new_fee);
        let result = entry.change_evm_fee_basis_points(new_fee);
        assert_eq!(result.evm_fee_basis_points, new_fee);
    }

    #[test]
    fn should_change_eth_fee_basis_points_via_dictionary() {
        let new_fee = 1337;
        let mut dictionary = get_sample_eth_evm_dictionary().unwrap();
        let eth_address = EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap());
        let fee_before = dictionary.get_eth_fee_basis_points(&eth_address).unwrap();
        assert_ne!(fee_before, new_fee);
        let updated_dictionary = dictionary.change_fee_basis_points(&eth_address, new_fee).unwrap();
        let result = updated_dictionary.get_eth_fee_basis_points(&eth_address).unwrap();
        assert_eq!(result, new_fee)
    }

    #[test]
    fn should_change_evm_fee_basis_points_via_dictionary() {
        let new_fee = 1337;
        let mut dictionary = get_sample_eth_evm_dictionary().unwrap();
        let evm_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let fee_before = dictionary.get_evm_fee_basis_points(&evm_address).unwrap();
        assert_ne!(fee_before, new_fee);
        let updated_dictionary = dictionary.change_fee_basis_points(&evm_address, new_fee).unwrap();
        let result = updated_dictionary.get_evm_fee_basis_points(&evm_address).unwrap();
        assert_eq!(result, new_fee)
    }

    #[test]
    fn should_change_fee_basis_points_and_update_in_db() {
        let db = get_test_database();
        let new_fee = 1337;
        let mut dictionary = get_sample_eth_evm_dictionary().unwrap();
        let eth_address = EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap());
        let fee_before = dictionary.get_eth_fee_basis_points(&eth_address).unwrap();
        assert_ne!(fee_before, new_fee);
        dictionary
            .change_fee_basis_points_and_update_in_db(&db, &eth_address, new_fee)
            .unwrap();
        let dictionary_from_db = EthEvmTokenDictionary::get_from_db(&db).unwrap();
        let result = dictionary_from_db.get_eth_fee_basis_points(&eth_address).unwrap();
        assert_eq!(result, new_fee)
    }

    #[test]
    fn should_set_last_withdrawal_timestamp_in_dictionary_entry() {
        let timestamp = get_unix_timestamp().unwrap();
        let human_readable_timestamp = get_last_withdrawal_date_as_human_readable_string(timestamp);
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let evm_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let entry = dictionary.get_entry_via_address(&evm_address).unwrap();
        let result = entry.set_last_withdrawal_timestamp(timestamp);
        assert_eq!(result.last_withdrawal, timestamp);
        assert_eq!(result.last_withdrawal_human_readable, human_readable_timestamp);
    }

    #[test]
    fn should_zero_accrued_fees_in_dictionary_entry() {
        let fees_before = U256::from(1337);
        let fees_after = U256::zero();
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let evm_address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let entry = dictionary.get_entry_via_address(&evm_address).unwrap();
        let updated_entry = entry.add_to_accrued_fees(fees_before);
        assert_eq!(updated_entry.accrued_fees, fees_before);
        let result = entry.zero_accrued_fees();
        assert_eq!(result.accrued_fees, fees_after);
    }

    #[test]
    fn should_set_last_withdrawal_timestamp_in_entry_via_dictionary() {
        let timestamp = get_unix_timestamp().unwrap();
        let mut dictionary = get_sample_eth_evm_dictionary().unwrap();
        let address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let entry_before = dictionary.get_entry_via_address(&address).unwrap();
        assert_eq!(entry_before.last_withdrawal, 0);
        let updated_dictionary = dictionary
            .set_last_withdrawal_timestamp_in_entry(&address, timestamp)
            .unwrap();
        let result = updated_dictionary.get_entry_via_address(&address).unwrap();
        assert_eq!(result.last_withdrawal, timestamp);
    }

    #[test]
    fn should_zero_accrued_fees_in_entry_via_dictionary() {
        let fees_before = U256::from(1337);
        let fees_after = U256::zero();
        let mut dictionary = get_sample_eth_evm_dictionary().unwrap();
        let address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let mut updated_dictionary = dictionary.increment_accrued_fee(&address, fees_before).unwrap();
        let entry = updated_dictionary.get_entry_via_address(&address).unwrap();
        assert_eq!(entry.accrued_fees, fees_before);
        let final_dictionary = updated_dictionary.zero_accrued_fees_in_entry(&address).unwrap();
        let result = final_dictionary.get_entry_via_address(&address).unwrap();
        assert_eq!(result.accrued_fees, fees_after);
    }

    #[test]
    fn should_get_fee_withdrawal_amount_via_dictionary() {
        let expected_fee = U256::from(1337);
        let mut dictionary = get_sample_eth_evm_dictionary().unwrap();
        let address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let updated_dictionary = dictionary.increment_accrued_fee(&address, expected_fee).unwrap();
        let result = updated_dictionary.get_fee_withdrawal_amount(&address).unwrap();
        assert_eq!(result, expected_fee);
    }

    #[test]
    fn should_withdraw_fees() {
        let timestamp = get_unix_timestamp().unwrap();
        let db = get_test_database();
        let expected_fee = U256::from(1337);
        let mut dictionary = get_sample_eth_evm_dictionary().unwrap();
        let expected_token_address =
            EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap());
        let address = EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let mut updated_dictionary = dictionary.increment_accrued_fee(&address, expected_fee).unwrap();
        let entry_before = updated_dictionary.get_entry_via_address(&address).unwrap();
        assert_eq!(entry_before.accrued_fees, expected_fee);
        assert_eq!(entry_before.last_withdrawal, 0);
        let (token_address, withdrawal_amount) = updated_dictionary.withdraw_fees(&db, &address).unwrap();
        assert_eq!(withdrawal_amount, expected_fee);
        assert_eq!(token_address, expected_token_address);
        let dictionary_from_db = EthEvmTokenDictionary::get_from_db(&db).unwrap();
        let entry_after = dictionary_from_db.get_entry_via_address(&address).unwrap();
        assert_eq!(entry_after.accrued_fees, U256::zero());
        assert!(entry_after.last_withdrawal >= timestamp);
    }

    fn get_pnt_address() -> EthAddress {
        EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap())
    }

    fn get_pnt_dictionary_entry() -> EthEvmTokenDictionaryEntry {
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        dictionary.get_entry_via_address(&get_pnt_address()).unwrap()
    }

    #[test]
    fn should_add_entry_and_update_in_db() {
        let db = get_test_database();
        let dictionary = EthEvmTokenDictionary::new(vec![]);
        let entry = get_pnt_dictionary_entry();
        dictionary.add_and_update_in_db(entry.clone(), &db).unwrap();
        let dictionary_from_db = EthEvmTokenDictionary::get_from_db(&db).unwrap();
        assert!(dictionary_from_db.contains(&entry));
    }

    #[test]
    fn should_remove_entry_via_eth_address_and_update_in_db() {
        let db = get_test_database();
        let dictionary = get_sample_eth_evm_dictionary().unwrap();
        let address = get_pnt_address();
        let entry = get_pnt_dictionary_entry();
        dictionary
            .remove_entry_via_eth_address_and_update_in_db(&address, &db)
            .unwrap();
        let dictionary_from_db = EthEvmTokenDictionary::get_from_db(&db).unwrap();
        assert!(!dictionary_from_db.contains(&entry));
    }
}
