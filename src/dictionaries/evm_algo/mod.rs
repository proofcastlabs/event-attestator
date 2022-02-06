pub(crate) mod dictionary;
pub(crate) mod dictionary_entry;

/*
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::eth_state::EthState,
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    dictionaries::dictionary_constants::ETH_EVM_DICTIONARY_KEY,
    fees::fee_utils::get_last_withdrawal_date_as_human_readable_string,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::{get_unix_timestamp, strip_hex_prefix},
};

pub(crate) mod test_utils;

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, DerefMut, Serialize, Deserialize)]
pub struct EvmAlgoTokenDictionary(pub Vec<EvmAlgoTokenDictionaryEntry>);

impl EvmAlgoTokenDictionary {
    pub fn convert_eth_amount_to_evm_amount(&self, address: &EthAddress, amount: U256) -> Result<U256> {
        self.get_entry_via_address(address)
            .and_then(|entry| entry.convert_eth_amount_to_evm_amount(amount))
    }

    pub fn convert_evm_amount_to_eth_amount(&self, address: &EthAddress, amount: U256) -> Result<U256> {
        self.get_entry_via_address(address)
            .and_then(|entry| entry.convert_evm_amount_to_eth_amount(amount))
    }

    pub fn to_json(&self) -> Result<EthEvmTokenDictionaryJson> {
        Ok(EthEvmTokenDictionaryJson::new(
            self.iter().map(|entry| entry.to_json()).collect(),
        ))
    }

    pub fn from_json(json: &EthEvmTokenDictionaryJson) -> Result<Self> {
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
        EthEvmTokenDictionaryJson::from_bytes(bytes).and_then(|json| Self::from_json(&json))
    }

    fn add(&self, entry: EvmAlgoTokenDictionaryEntry) -> Self {
        let mut new_self = self.clone();
        match self.contains(&entry) {
            true => {
                info!("✘ Not adding new `EvmAlgoTokenDictionaryEntry` ∵ entry already extant!");
                new_self
            },
            false => {
                info!("✔ Adding `EvmAlgoTokenDictionary` entry: {:?}...", entry);
                new_self.push(entry);
                new_self
            },
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
                info!("✘ No `EthEvmTokenDictionaryJson` in db! Initializing a new one...");
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

    pub fn remove_entry_via_eth_address_and_update_in_db<D: DatabaseInterface>(
        &self,
        eth_address: &EthAddress,
        db: &D,
    ) -> Result<()> {
        self.get_entry_via_eth_address(eth_address)
            .and_then(|entry| self.remove_and_update_in_db(&entry, db))
    }

    pub fn get_entry_via_eth_address(&self, address: &EthAddress) -> Result<EvmAlgoTokenDictionaryEntry> {
        match self.iter().find(|entry| entry.eth_address == *address) {
            Some(entry) => Ok(entry.clone()),
            None => Err(format!("No `EvmAlgoTokenDictionaryEntry` exists with ETH address: {}", address).into()),
        }
    }

    pub fn get_entry_via_evm_address(&self, address: &EthAddress) -> Result<EvmAlgoTokenDictionaryEntry> {
        match self.iter().find(|entry| &entry.evm_address == address) {
            Some(entry) => Ok(entry.clone()),
            None => Err(format!("No `EvmAlgoTokenDictionaryEntry` exists with ETH address: {}", address).into()),
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
                .map(|ref entry_json| EvmAlgoTokenDictionaryEntry::from_json(entry_json))
                .collect::<Result<Vec<EvmAlgoTokenDictionaryEntry>>>()?,
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

    fn get_entry_via_address(&self, address: &EthAddress) -> Result<EvmAlgoTokenDictionaryEntry> {
        self.get_entry_via_eth_address(address)
            .or_else(|_| self.get_entry_via_evm_address(address))
    }

    pub fn replace_entry(
        &self,
        entry_to_remove: &EvmAlgoTokenDictionaryEntry,
        entry_to_add: EvmAlgoTokenDictionaryEntry,
    ) -> Self {
        if entry_to_add == *entry_to_remove {
            info!("✘ Entry to replace is identical to new entry, doing nothing!");
            self.clone()
        } else {
            info!("✔ Replacing dictionary entry...");
            self.add(entry_to_add).remove(entry_to_remove)
        }
    }

    pub fn increment_accrued_fee(&self, address: &EthAddress, addend: U256) -> Result<Self> {
        self.get_entry_via_address(address)
            .map(|entry| self.replace_entry(&entry, entry.add_to_accrued_fees(addend)))
    }

    fn set_accrued_fee(&self, address: &EthAddress, fee: U256) -> Result<Self> {
        self.get_entry_via_eth_address(address)
            .map(|entry| self.replace_entry(&entry, entry.set_accrued_fees(fee)))
    }

    pub fn set_accrued_fees_and_save_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        address: &EthAddress,
        fee: U256,
    ) -> Result<()> {
        self.set_accrued_fee(address, fee)
            .and_then(|new_dictionary| new_dictionary.save_in_db(db))
    }

    pub fn increment_accrued_fees(&self, fee_tuples: Vec<(EthAddress, U256)>) -> Result<Self> {
        info!("✔ Incrementing accrued fees...");
        fee_tuples
            .iter()
            .filter(|(address, addend)| {
                if *addend > U256::zero() {
                    true
                } else {
                    info!("✘ Not adding to accrued fees for {} ∵ increment is 0!", address);
                    false
                }
            })
            .try_fold(self.clone(), |new_self, (address, addend)| {
                new_self.increment_accrued_fee(address, *addend)
            })
    }

    pub fn increment_accrued_fees_and_save_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        fee_tuples: Vec<(EthAddress, U256)>,
    ) -> Result<()> {
        self.increment_accrued_fees(fee_tuples)
            .and_then(|new_dictionary| new_dictionary.save_in_db(db))
    }

    fn change_eth_fee_basis_points(&self, eth_address: &EthAddress, new_fee: u64) -> Result<Self> {
        info!(
            "✔ Changing ETH fee basis points for address {} to {}...",
            eth_address, new_fee
        );
        self.get_entry_via_eth_address(eth_address)
            .map(|entry| self.replace_entry(&entry, entry.change_eth_fee_basis_points(new_fee)))
    }

    fn change_evm_fee_basis_points(&self, evm_address: &EthAddress, new_fee: u64) -> Result<Self> {
        info!(
            "✔ Changing EVM fee basis points for address {} to {}...",
            evm_address, new_fee
        );
        self.get_entry_via_evm_address(evm_address)
            .map(|entry| self.replace_entry(&entry, entry.change_evm_fee_basis_points(new_fee)))
    }

    fn change_fee_basis_points(&self, address: &EthAddress, new_fee: u64) -> Result<Self> {
        self.change_eth_fee_basis_points(address, new_fee)
            .or_else(|_| self.change_evm_fee_basis_points(address, new_fee))
    }

    pub fn change_fee_basis_points_and_update_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        address: &EthAddress,
        new_fee: u64,
    ) -> Result<()> {
        self.change_fee_basis_points(address, new_fee)
            .and_then(|updated_dictionary| updated_dictionary.save_in_db(db))
    }

    fn set_last_withdrawal_timestamp_in_entry(&self, address: &EthAddress, timestamp: u64) -> Result<Self> {
        self.get_entry_via_address(address)
            .map(|entry| self.replace_entry(&entry, entry.set_last_withdrawal_timestamp(timestamp)))
    }

    fn zero_accrued_fees_in_entry(&self, address: &EthAddress) -> Result<Self> {
        self.get_entry_via_address(address)
            .map(|entry| self.replace_entry(&entry, entry.zero_accrued_fees()))
    }

    pub fn withdraw_fees_and_save_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        maybe_entry_address: &EthAddress,
    ) -> Result<(EthAddress, U256)> {
        let entry = self.get_entry_via_address(maybe_entry_address)?;
        let token_address = entry.eth_address;
        let withdrawal_amount = entry.accrued_fees;
        self.set_last_withdrawal_timestamp_in_entry(&token_address, get_unix_timestamp()?)
            .and_then(|dictionary| dictionary.zero_accrued_fees_in_entry(&token_address))
            .and_then(|dictionary| dictionary.save_in_db(db))
            .map(|_| (token_address, withdrawal_amount))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct EthEvmTokenDictionaryJson(pub Vec<EthEvmTokenDictionaryEntryJson>);

impl EthEvmTokenDictionaryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EvmAlgoTokenDictionaryEntry {
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
    pub eth_token_decimals: Option<u16>,
    pub evm_token_decimals: Option<u16>,
}

impl EvmAlgoTokenDictionaryEntry {
    fn require_decimal_conversion(&self) -> bool {
        self.eth_token_decimals.is_some()
            && self.evm_token_decimals.is_some()
            && self.eth_token_decimals != self.evm_token_decimals
    }

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
            eth_token_decimals: self.eth_token_decimals,
            evm_token_decimals: self.evm_token_decimals,
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
            eth_token_decimals: json.eth_token_decimals,
            evm_token_decimals: json.evm_token_decimals,
        })
    }

    fn set_accrued_fees(&self, fee: U256) -> Self {
        info!("✔ Setting accrued fees to {}...", fee);
        let mut new_entry = self.clone();
        new_entry.accrued_fees = fee;
        new_entry.accrued_fees_human_readable = fee.as_u128();
        new_entry
    }

    pub fn from_str(json_string: &str) -> Result<Self> {
        EthEvmTokenDictionaryEntryJson::from_str(json_string).and_then(|entry_json| Self::from_json(&entry_json))
    }

    pub fn add_to_accrued_fees(&self, addend: U256) -> Self {
        let new_accrued_fees = self.accrued_fees + addend;
        info!("✔ Adding to accrued fees in {:?}...", self);
        info!(
            "✔ Updating accrued fees from {} to {}...",
            self.accrued_fees, new_accrued_fees
        );
        let mut new_entry = self.clone();
        new_entry.accrued_fees = new_accrued_fees;
        new_entry.accrued_fees_human_readable = new_accrued_fees.as_u128();
        new_entry
    }

    pub fn change_eth_fee_basis_points(&self, new_fee: u64) -> Self {
        info!(
            "✔ Changing ETH fee basis points for address {} from {} to {}...",
            self.eth_address, self.eth_fee_basis_points, new_fee
        );
        let mut new_entry = self.clone();
        new_entry.eth_fee_basis_points = new_fee;
        new_entry
    }

    pub fn change_evm_fee_basis_points(&self, new_fee: u64) -> Self {
        info!(
            "✔ Changing EVM fee basis points for address {} from {} to {}...",
            self.evm_address, self.evm_fee_basis_points, new_fee
        );
        let mut new_entry = self.clone();
        new_entry.evm_fee_basis_points = new_fee;
        new_entry
    }

    fn set_last_withdrawal_timestamp(&self, timestamp: u64) -> Self {
        let timestamp_human_readable = get_last_withdrawal_date_as_human_readable_string(timestamp);
        info!("✔ Setting withdrawal date to {}", timestamp_human_readable);
        let mut new_entry = self.clone();
        new_entry.last_withdrawal = timestamp;
        new_entry.last_withdrawal_human_readable = timestamp_human_readable;
        new_entry
    }

    fn zero_accrued_fees(&self) -> Self {
        info!("✔ Zeroing accrued fees in {:?}...", self);
        let mut new_entry = self.clone();
        new_entry.accrued_fees = U256::zero();
        new_entry.accrued_fees_human_readable = 0;
        new_entry
    }

    fn get_eth_token_decimals(&self) -> Result<u16> {
        self.eth_token_decimals
            .ok_or_else(|| format!("Dictionary entry does NOT have ETH token decimals set! {:?}", self).into())
    }

    fn get_evm_token_decimals(&self) -> Result<u16> {
        self.evm_token_decimals
            .ok_or_else(|| format!("Dictionary entry does NOT have EVM token decimals set! {:?}", self).into())
    }

    pub fn convert_eth_amount_to_evm_amount(&self, amount: U256) -> Result<U256> {
        info!("✔ Converting from ETH amount to EVM amount...");
        self.convert_amount(amount, true)
    }

    pub fn convert_evm_amount_to_eth_amount(&self, amount: U256) -> Result<U256> {
        info!("✔ Converting from EVM amount to ETH amount...");
        self.convert_amount(amount, false)
    }

    fn convert_amount(&self, amount: U256, eth_to_evm: bool) -> Result<U256> {
        if self.require_decimal_conversion() {
            let eth_token_decimals = self.get_eth_token_decimals()?;
            let evm_token_decimals = self.get_evm_token_decimals()?;
            let to = if eth_to_evm {
                evm_token_decimals
            } else {
                eth_token_decimals
            };
            let from = if eth_to_evm {
                eth_token_decimals
            } else {
                evm_token_decimals
            };
            let multiplicand = U256::from(10).pow(U256::from(to));
            let divisor = U256::from(10).pow(U256::from(from));
            info!("✔ Converting {} from {} decimals to {}...", amount, from, to);
            Ok((amount * multiplicand) / divisor)
        } else {
            info!(
                "✔ Amounts for this dictionary entry do NOT require decimal conversion! {:?}",
                self,
            );
            Ok(amount)
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
    eth_token_decimals: Option<u16>,
    evm_token_decimals: Option<u16>,
}

impl EthEvmTokenDictionaryEntryJson {
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

pub fn get_eth_evm_token_dictionary_from_db_and_add_to_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Getting `EvmAlgoTokenDictionary` and adding to ETH state...");
    EvmAlgoTokenDictionary::get_from_db(state.db).and_then(|dictionary| state.add_eth_evm_token_dictionary(dictionary))
}

#[cfg(test)]
mod tests {
    use super::*;
}
*/
