use std::{cmp::Ordering, str::FromStr};

use derive_more::{Constructor, Deref, DerefMut};
use eos_chain::AccountName as EosAccountName;
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

pub mod test_utils;

use crate::{
    chains::eos::eos_utils::remove_symbol_from_eos_asset,
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    dictionaries::dictionary_constants::EOS_ETH_DICTIONARY_KEY,
    errors::AppError,
    fees::fee_utils::get_last_withdrawal_date_as_human_readable_string,
    state::EosState,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::{
        get_unix_timestamp,
        left_pad_with_zeroes,
        right_pad_or_truncate,
        right_pad_with_zeroes,
        strip_hex_prefix,
        truncate_str,
    },
};

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, DerefMut)]
pub struct EosEthTokenDictionary(pub Vec<EosEthTokenDictionaryEntry>);

impl EosEthTokenDictionary {
    pub fn increment_accrued_fee(&self, address: &EthAddress, addend: U256) -> Result<Self> {
        self.get_entry_via_eth_address(address)
            .map(|entry| self.replace_entry(&entry, entry.add_to_accrued_fees(addend)))
    }

    fn set_accrued_fee(&self, address: &EthAddress, fee: U256) -> Result<Self> {
        self.get_entry_via_eth_address(address)
            .map(|entry| self.replace_entry(&entry, entry.set_accrued_fees(fee)))
    }

    pub fn increment_accrued_fees(&self, fee_tuples: &[(EthAddress, U256)]) -> Result<Self> {
        info!("✔ Incrementing accrued fees...");
        fee_tuples
            .iter()
            .filter(|(address, addend)| {
                if addend.is_zero() {
                    info!("✘ Not adding to accrued fees for {} ∵ increment is 0!", address);
                    false
                } else {
                    true
                }
            })
            .try_fold(self.clone(), |new_self, (address, addend)| {
                new_self.increment_accrued_fee(address, *addend)
            })
    }

    pub fn increment_accrued_fees_and_save_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        fee_tuples: &[(EthAddress, U256)],
    ) -> Result<()> {
        self.increment_accrued_fees(fee_tuples)
            .and_then(|new_dictionary| new_dictionary.save_to_db(db))
    }

    pub fn set_accrued_fees_and_save_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        address: &EthAddress,
        fee: U256,
    ) -> Result<()> {
        self.set_accrued_fee(address, fee)
            .and_then(|new_dictionary| new_dictionary.save_to_db(db))
    }

    pub fn change_eth_fee_basis_points_and_update_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        address: &EthAddress,
        new_fee: u64,
    ) -> Result<()> {
        self.change_eth_fee_basis_points(address, new_fee)
            .and_then(|updated_dictionary| updated_dictionary.save_to_db(db))
    }

    pub fn change_eos_fee_basis_points_and_update_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        address: &EosAccountName,
        new_fee: u64,
    ) -> Result<()> {
        self.change_eos_fee_basis_points(address, new_fee)
            .and_then(|updated_dictionary| updated_dictionary.save_to_db(db))
    }

    fn set_last_withdrawal_timestamp_in_entry(&self, address: &EthAddress, timestamp: u64) -> Result<Self> {
        self.get_entry_via_eth_address(address)
            .map(|entry| self.replace_entry(&entry, entry.set_last_withdrawal_timestamp(timestamp)))
    }

    fn zero_accrued_fees_in_entry(&self, address: &EthAddress) -> Result<Self> {
        self.get_entry_via_eth_address(address).map(|entry| {
            info!("✔ Zeroing accrued fees in entry...");
            self.replace_entry(&entry, entry.zero_accrued_fees())
        })
    }

    pub fn withdraw_fees_and_save_in_db<D: DatabaseInterface>(
        &self,
        db: &D,
        maybe_entry_address: &EthAddress,
    ) -> Result<(EthAddress, U256)> {
        let entry = self.get_entry_via_eth_address(maybe_entry_address)?;
        let token_address = entry.eth_address;
        let withdrawal_amount = entry.accrued_fees;
        self.set_last_withdrawal_timestamp_in_entry(&token_address, get_unix_timestamp()?)
            .and_then(|dictionary| dictionary.zero_accrued_fees_in_entry(&token_address))
            .and_then(|dictionary| dictionary.save_to_db(db))
            .map(|_| (token_address, withdrawal_amount))
    }

    fn change_eth_fee_basis_points(&self, eth_address: &EthAddress, new_fee: u64) -> Result<Self> {
        info!(
            "✔ Changing ETH fee basis points for address {} to {}...",
            eth_address, new_fee
        );
        self.get_entry_via_eth_address(eth_address)
            .map(|entry| self.replace_entry(&entry, entry.change_eth_fee_basis_points(new_fee)))
    }

    fn change_eos_fee_basis_points(&self, eos_address: &EosAccountName, new_fee: u64) -> Result<Self> {
        info!(
            "✔ Changing EOS fee basis points for address {} to {}...",
            eos_address, new_fee
        );
        self.get_entry_via_eos_address(eos_address)
            .map(|entry| self.replace_entry(&entry, entry.change_eos_fee_basis_points(new_fee)))
    }

    pub fn replace_entry(
        &self,
        entry_to_remove: &EosEthTokenDictionaryEntry,
        entry_to_add: EosEthTokenDictionaryEntry,
    ) -> Self {
        if entry_to_add == *entry_to_remove {
            info!("✘ Entry to replace is identical to new entry, doing nothing!");
            self.clone()
        } else {
            info!("✔ Replacing dictionary entry...");
            self.add(entry_to_add).remove(entry_to_remove)
        }
    }

    pub fn get_eth_fee_basis_points(&self, eth_address: &EthAddress) -> Result<u64> {
        Ok(self.get_entry_via_eth_address(eth_address)?.eth_fee_basis_points)
    }

    pub fn get_eos_fee_basis_points(&self, eos_address: &EosAccountName) -> Result<u64> {
        Ok(self.get_entry_via_eos_address(eos_address)?.eos_fee_basis_points)
    }

    pub fn to_json(&self) -> Result<EosEthTokenDictionaryJson> {
        Ok(EosEthTokenDictionaryJson::new(
            self.iter().map(|entry| entry.to_json()).collect(),
        ))
    }

    pub fn from_json(json: &EosEthTokenDictionaryJson) -> Result<Self> {
        Ok(Self(
            json.iter()
                .map(EosEthTokenDictionaryEntry::from_json)
                .collect::<Result<Vec<EosEthTokenDictionaryEntry>>>()?,
        ))
    }

    fn to_bytes(&self) -> Result<Bytes> {
        self.to_json()?.to_bytes()
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        EosEthTokenDictionaryJson::from_bytes(bytes).and_then(|json| Self::from_json(&json))
    }

    fn add(&self, entry: EosEthTokenDictionaryEntry) -> Self {
        let mut new_self = self.clone();
        info!("✔ Adding `EosEthTokenDictionary` entry: {:?}...", entry);
        match self.contains(&entry) {
            true => {
                info!("Not adding new `EosEthTokenDictionaryEntry` ∵ account name already extant!");
                new_self
            },
            false => {
                new_self.push(entry);
                new_self
            },
        }
    }

    fn remove(mut self, entry: &EosEthTokenDictionaryEntry) -> Self {
        info!("✔ Removing `EosEthTokenDictionary` entry: {:?}...", entry);
        match self.contains(entry) {
            false => self,
            true => {
                info!("Removing `EosEthTokenDictionaryEntry`: {:?}", entry);
                self.retain(|x| x != entry);
                self
            },
        }
    }

    pub fn save_to_db<D: DatabaseInterface>(&self, db: &D) -> Result<()> {
        db.put(
            EOS_ETH_DICTIONARY_KEY.to_vec(),
            self.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_from_db<D: DatabaseInterface>(db: &D) -> Result<Self> {
        info!("✔ Getting `EosEthTokenDictionaryJson` from db...");
        match db.get(EOS_ETH_DICTIONARY_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(_) => {
                info!("✔ No `EosEthTokenDictionaryJson` in db! Initializing a new one...");
                Ok(Self::new(vec![]))
            },
        }
    }

    pub fn add_and_update_in_db<D: DatabaseInterface>(self, entry: EosEthTokenDictionaryEntry, db: &D) -> Result<Self> {
        let new_self = self.add(entry);
        new_self.save_to_db(db)?;
        Ok(new_self)
    }

    fn remove_and_update_in_db<D: DatabaseInterface>(self, entry: &EosEthTokenDictionaryEntry, db: &D) -> Result<Self> {
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

    pub fn get_entry_via_eth_address(&self, address: &EthAddress) -> Result<EosEthTokenDictionaryEntry> {
        match self.iter().find(|entry| &entry.eth_address == address) {
            Some(entry) => Ok(entry.clone()),
            None => Err(format!("No `EosEthTokenDictionaryEntry` exists with ETH address: {}", address).into()),
        }
    }

    pub fn get_entry_via_eos_address(&self, eos_address: &EosAccountName) -> Result<EosEthTokenDictionaryEntry> {
        info!("✔ Getting dictionary entry via EOS token address: {}", eos_address);
        match self.iter().find(|entry| &entry.eos_address == eos_address) {
            Some(entry) => Ok(entry.clone()),
            None => Err(format!(
                "No `EosEthTokenDictionaryEntry` exists with EOS address: {}",
                eos_address
            )
            .into()),
        }
    }

    pub fn get_entries_via_eos_symbol(&self, eos_symbol: &str) -> Vec<EosEthTokenDictionaryEntry> {
        info!("✔ Getting dictionary entry via EOS symbol: {}", eos_symbol);
        self.iter()
            .filter(|entry| entry.eos_symbol == eos_symbol)
            .cloned()
            .collect()
    }

    pub fn get_entry_via_eos_address_symbol_and_decimals(
        &self,
        eos_address: &EosAccountName,
        eos_symbol: &str,
        num_decimals: usize,
    ) -> Result<EosEthTokenDictionaryEntry> {
        info!("✔ Getting dictionary entry via EOS token address, symbol & decimals...");
        self.get_entry_via_eos_address(eos_address).and_then(|entry| {
            if entry.eos_symbol == eos_symbol {
                if entry.eos_token_decimals == num_decimals {
                    Ok(entry)
                } else {
                    Err(format!(
                        "No `EosEthTokenDictionaryEntry` exists with num decimals: {}",
                        num_decimals
                    )
                    .into())
                }
            } else {
                Err(format!("No `EosEthTokenDictionaryEntry` exists with EOS symbol: {}", eos_symbol).into())
            }
        })
    }

    pub fn get_entry_via_eos_address_and_symbol(
        &self,
        eos_symbol: &str,
        eos_address: &str,
    ) -> Result<EosEthTokenDictionaryEntry> {
        info!("✔ Getting dictionary entry via EOS token address, symbol & decimals...");
        let entries = self
            .get_entries_via_eos_symbol(eos_symbol)
            .iter()
            .filter(|entry| entry.eos_address == eos_address)
            .cloned()
            .collect::<Vec<EosEthTokenDictionaryEntry>>();
        if entries.is_empty() {
            Err(format!("No entry with EOS symbol {} & address {}!", eos_symbol, eos_address).into())
        } else if entries.len() > 1 {
            debug!("> than 1 entry found: {:?}", entries);
            Err(format!(
                "Found > 1 entries with EOS symbol {} & address {}!",
                eos_symbol, eos_address
            )
            .into())
        } else {
            Ok(entries[0].clone())
        }
    }

    pub fn get_eos_account_name_from_eth_token_address(&self, address: &EthAddress) -> Result<String> {
        self.get_entry_via_eth_address(address).map(|entry| entry.eos_address)
    }

    pub fn get_eth_address_via_eos_address(&self, eos_address: &EosAccountName) -> Result<EthAddress> {
        self.get_entry_via_eos_address(eos_address)
            .map(|entry| entry.eth_address)
    }

    pub fn is_token_supported(&self, address: &EthAddress) -> bool {
        self.get_eos_account_name_from_eth_token_address(address).is_ok()
    }

    pub fn to_eth_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|entry| entry.eth_address).collect()
    }

    pub fn to_eos_accounts(&self) -> Result<Vec<EosAccountName>> {
        self.iter()
            .map(|entry| Ok(EosAccountName::from_str(&entry.eos_address)?))
            .collect()
    }

    pub fn to_unique_eos_accounts(&self) -> Result<Vec<EosAccountName>> {
        let mut accounts = self.to_eos_accounts()?;
        accounts.sort();
        accounts.dedup();
        Ok(accounts)
    }

    pub fn convert_u256_to_eos_asset_string(&self, address: &EthAddress, eth_amount: &U256) -> Result<String> {
        self.get_entry_via_eth_address(address)
            .and_then(|entry| entry.convert_u256_to_eos_asset_string(eth_amount))
    }

    pub fn get_zero_eos_asset_amount_via_eth_token_address(&self, eth_address: &EthAddress) -> Result<String> {
        self.get_entry_via_eth_address(eth_address)
            .map(|entry| entry.get_zero_eos_asset())
    }
}

impl FromStr for EosEthTokenDictionary {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        let entry_jsons: Vec<EosEthTokenDictionaryEntryJson> = serde_json::from_str(s)?;
        Ok(Self::new(
            entry_jsons
                .iter()
                .map(EosEthTokenDictionaryEntry::from_json)
                .collect::<Result<Vec<EosEthTokenDictionaryEntry>>>()?,
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosEthTokenDictionaryJson(pub Vec<EosEthTokenDictionaryEntryJson>);

impl EosEthTokenDictionaryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EosEthTokenDictionaryEntry {
    pub eth_token_decimals: usize,
    pub eos_token_decimals: usize,
    pub eos_symbol: String,
    pub eth_symbol: String,
    pub eos_address: String,
    pub eth_address: EthAddress,
    pub eth_fee_basis_points: u64,
    pub eos_fee_basis_points: u64,
    pub accrued_fees: U256,
    pub last_withdrawal: u64,
    pub accrued_fees_human_readable: u128,
    pub last_withdrawal_human_readable: String,
}

impl FromStr for EosEthTokenDictionaryEntry {
    type Err = AppError;

    fn from_str(json_string: &str) -> Result<Self> {
        EosEthTokenDictionaryEntryJson::from_str(json_string).and_then(|entry_json| Self::from_json(&entry_json))
    }
}

impl EosEthTokenDictionaryEntry {
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

    fn set_accrued_fees(&self, fee: U256) -> Self {
        info!("✔ Setting accrued fees to {}...", fee);
        let mut new_entry = self.clone();
        new_entry.accrued_fees = fee;
        new_entry.accrued_fees_human_readable = fee.as_u128();
        new_entry
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

    pub fn change_eos_fee_basis_points(&self, new_fee: u64) -> Self {
        info!(
            "✔ Changing EOS fee basis points for address {} from {} to {}...",
            self.eos_address, self.eos_fee_basis_points, new_fee
        );
        let mut new_entry = self.clone();
        new_entry.eos_fee_basis_points = new_fee;
        new_entry
    }

    fn to_json(&self) -> EosEthTokenDictionaryEntryJson {
        EosEthTokenDictionaryEntryJson {
            eth_token_decimals: self.eth_token_decimals,
            eos_token_decimals: self.eos_token_decimals,
            eos_symbol: self.eos_symbol.to_string(),
            eth_symbol: self.eth_symbol.to_string(),
            eos_address: self.eos_address.to_string(),
            eth_address: hex::encode(self.eth_address),
            eth_fee_basis_points: Some(self.eth_fee_basis_points),
            eos_fee_basis_points: Some(self.eos_fee_basis_points),
            accrued_fees: Some(self.accrued_fees.as_u128()),
            last_withdrawal: Some(self.last_withdrawal),
        }
    }

    pub fn from_json(json: &EosEthTokenDictionaryEntryJson) -> Result<Self> {
        let timestamp = json.last_withdrawal.unwrap_or_default();
        let accrued_fees = U256::from(json.accrued_fees.unwrap_or_default());
        Ok(Self {
            eth_token_decimals: json.eth_token_decimals,
            eos_token_decimals: json.eos_token_decimals,
            eos_symbol: json.eos_symbol.to_string(),
            eth_symbol: json.eth_symbol.to_string(),
            eos_address: json.eos_address.to_string(),
            eth_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.eth_address))?),
            eth_fee_basis_points: json.eth_fee_basis_points.unwrap_or_default(),
            eos_fee_basis_points: json.eos_fee_basis_points.unwrap_or_default(),
            accrued_fees_human_readable: accrued_fees.as_u128(),
            last_withdrawal: timestamp,
            last_withdrawal_human_readable: get_last_withdrawal_date_as_human_readable_string(timestamp),
            accrued_fees,
        })
    }

    fn get_decimal_and_fractional_parts_of_eos_asset(eos_asset: &str) -> (&str, &str) {
        let parts = remove_symbol_from_eos_asset(eos_asset)
            .split('.')
            .collect::<Vec<&str>>();
        let decimal_part = parts[0];
        let fractional_part = if parts.len() > 1 { parts[1] } else { "" };
        (decimal_part, fractional_part)
    }

    pub fn convert_eos_asset_to_eth_amount(&self, eos_asset: &str) -> Result<U256> {
        info!("✔ Convert EOS asset to ETH amount...");
        let (decimal_str, fraction_str) = Self::get_decimal_and_fractional_parts_of_eos_asset(eos_asset);
        let num_decimals = fraction_str.len();
        let expected_num_decimals = self.eos_token_decimals;
        if num_decimals != expected_num_decimals {
            return Err(format!(
                "Expected {} decimals in EOS asset, found {}! ",
                expected_num_decimals, num_decimals
            )
            .into());
        }
        let augmented_fraction_str = match self.eth_token_decimals.cmp(&self.eos_token_decimals) {
            Ordering::Greater => right_pad_with_zeroes(fraction_str, self.eth_token_decimals),
            Ordering::Equal => fraction_str.to_string(),
            Ordering::Less => truncate_str(fraction_str, self.eos_token_decimals - self.eth_token_decimals).to_string(),
        };
        Ok(U256::from_dec_str(&format!(
            "{}{}",
            decimal_str, augmented_fraction_str
        ))?)
    }

    pub fn convert_u256_to_eos_asset_string(&self, amount: &U256) -> Result<String> {
        let amount_str = amount.to_string();
        match amount_str.len().cmp(&self.eth_token_decimals) {
            Ordering::Greater | Ordering::Equal => {
                let decimal_point_index = amount_str.len() - self.eth_token_decimals;
                let (decimal_str, fraction_str) = &amount_str.split_at(decimal_point_index);
                let augmented_fraction_str = right_pad_or_truncate(fraction_str, self.eos_token_decimals);
                let augmented_decimal_str = if decimal_str == &"" { "0" } else { decimal_str };
                Ok(format!(
                    "{}.{} {}",
                    augmented_decimal_str,
                    augmented_fraction_str,
                    self.eos_symbol.to_uppercase()
                ))
            },
            Ordering::Less => {
                let fraction_str = left_pad_with_zeroes(&amount_str, self.eth_token_decimals);
                let augmented_fraction_str = right_pad_or_truncate(&fraction_str, self.eos_token_decimals);
                Ok(format!(
                    "0.{} {}",
                    augmented_fraction_str,
                    self.eos_symbol.to_uppercase()
                ))
            },
        }
    }

    pub fn convert_u64_to_eos_asset(&self, u_64: u64) -> String {
        let amount_str = u_64.to_string();
        match amount_str.len().cmp(&self.eos_token_decimals) {
            Ordering::Equal => format!("0.{} {}", amount_str, self.eos_symbol),
            Ordering::Less => {
                let fraction_part = left_pad_with_zeroes(&amount_str, self.eos_token_decimals);
                format!("0.{} {}", fraction_part, self.eos_symbol)
            },
            Ordering::Greater => {
                let (decimal_part, fraction_part) = &amount_str.split_at(amount_str.len() - self.eos_token_decimals);
                format!("{}.{} {}", decimal_part, fraction_part, self.eos_symbol)
            },
        }
    }

    pub fn get_zero_eos_asset(&self) -> String {
        self.convert_u64_to_eos_asset(0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EosEthTokenDictionaryEntryJson {
    pub eth_token_decimals: usize,
    pub eos_token_decimals: usize,
    pub eth_symbol: String,
    pub eos_symbol: String,
    pub eth_address: String,
    pub eos_address: String,
    pub eth_fee_basis_points: Option<u64>,
    pub eos_fee_basis_points: Option<u64>,
    pub accrued_fees: Option<u128>,
    pub last_withdrawal: Option<u64>,
}

impl FromStr for EosEthTokenDictionaryEntryJson {
    type Err = AppError;

    fn from_str(json_string: &str) -> Result<Self> {
        match serde_json::from_str(json_string) {
            Ok(result) => Ok(result),
            Err(err) => Err(err.into()),
        }
    }
}

pub fn get_eos_eth_token_dictionary_from_db_and_add_to_eos_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Getting `EosERc20Dictionary` and adding to EOS state...");
    EosEthTokenDictionary::get_from_db(state.db).and_then(|dictionary| state.add_eos_eth_token_dictionary(dictionary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dictionaries::eos_eth::test_utils::{
            get_sample_eos_eth_token_dictionary,
            get_sample_eos_eth_token_dictionary_entry_1,
            get_sample_eos_eth_token_dictionary_entry_2,
            get_sample_eos_eth_token_dictionary_entry_3,
            get_sample_eos_eth_token_dictionary_json,
        },
        errors::AppError,
        test_utils::get_test_database,
    };

    #[test]
    fn eos_eth_token_dictionary_should_contain_eos_eth_token_dictionary_entry() {
        let dictionary_entry = get_sample_eos_eth_token_dictionary_entry_1();
        let dictionary = get_sample_eos_eth_token_dictionary();
        assert!(dictionary.contains(&dictionary_entry));
    }

    #[test]
    fn eos_eth_token_dictionary_should_no_contain_other_eos_eth_token_dictionary_entry() {
        let token_address_hex = "9e57CB2a4F462a5258a49E88B4331068a391DE66".to_string();
        let eth_basis_points = 0;
        let eos_basis_points = 0;
        let other_dictionary_entry = EosEthTokenDictionaryEntry::new(
            18,
            9,
            "SYM".to_string(),
            "SYM".to_string(),
            "SampleTokenx".to_string(),
            EthAddress::from_slice(&hex::decode(token_address_hex).unwrap()),
            eth_basis_points,
            eos_basis_points,
            U256::zero(),
            0,
            0,
            "".to_string(),
        );
        let dictionary = get_sample_eos_eth_token_dictionary();
        assert!(!dictionary.contains(&other_dictionary_entry));
    }

    #[test]
    fn should_push_into_eos_eth_token_dictionary_if_entry_not_extant() {
        let expected_num_entries_before = 1;
        let expected_num_entries_after = 2;
        let dictionary_entries = EosEthTokenDictionary::new(vec![get_sample_eos_eth_token_dictionary_entry_1()]);
        assert_eq!(dictionary_entries.len(), expected_num_entries_before);
        let updated_dictionary = dictionary_entries.add(get_sample_eos_eth_token_dictionary_entry_2());
        assert_eq!(updated_dictionary.len(), expected_num_entries_after);
    }

    #[test]
    fn should_not_push_into_eos_eth_token_dictionary_if_entry_extant() {
        let expected_num_account_names = 3;
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        assert_eq!(dictionary_entries.len(), expected_num_account_names);
        let updated_dictionary = dictionary_entries.add(get_sample_eos_eth_token_dictionary_entry_1());
        assert_eq!(updated_dictionary.len(), expected_num_account_names);
    }

    #[test]
    fn should_remove_entry_from_eos_eth_token_dictionary() {
        let expected_num_entries_before = 3;
        let expected_num_entries_after = 2;
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        assert_eq!(dictionary_entries.len(), expected_num_entries_before);
        let updated_dictionary = dictionary_entries.remove(&get_sample_eos_eth_token_dictionary_entry_2());
        assert_eq!(updated_dictionary.len(), expected_num_entries_after);
    }

    #[test]
    fn should_savee_eos_eth_token_dictionary_in_db() {
        let db = get_test_database();
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        dictionary_entries.save_to_db(&db).unwrap();
        let result = db
            .get(EOS_ETH_DICTIONARY_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .unwrap();
        assert_eq!(result, dictionary_entries.to_bytes().unwrap());
    }

    #[test]
    fn get_from_db_should_get_empty_eos_eth_token_dictionary_if_non_extant() {
        let db = get_test_database();
        let expected_result = EosEthTokenDictionary::new(vec![]);
        let result = EosEthTokenDictionary::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_from_db_should_get_correct_eos_eth_token_dictionary_if_extant() {
        let db = get_test_database();
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        dictionary_entries.save_to_db(&db).unwrap();
        let result = EosEthTokenDictionary::get_from_db(&db).unwrap();
        assert_eq!(result, dictionary_entries);
    }

    #[test]
    fn eos_eth_token_dictionary_should_add_new_entry_and_update_in_db() {
        let db = get_test_database();
        let dictionary_entries = EosEthTokenDictionary::new(vec![
            get_sample_eos_eth_token_dictionary_entry_1(),
            get_sample_eos_eth_token_dictionary_entry_2(),
        ]);
        dictionary_entries
            .add_and_update_in_db(get_sample_eos_eth_token_dictionary_entry_3(), &db)
            .unwrap();
        let result = EosEthTokenDictionary::get_from_db(&db).unwrap();
        assert_eq!(result, get_sample_eos_eth_token_dictionary());
    }

    #[test]
    fn eos_eth_token_dictionary_should_remove_entry_and_update_in_db() {
        let db = get_test_database();
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        dictionary_entries.save_to_db(&db).unwrap();
        dictionary_entries
            .remove_and_update_in_db(&get_sample_eos_eth_token_dictionary_entry_1(), &db)
            .unwrap();
        let result = EosEthTokenDictionary::get_from_db(&db).unwrap();
        let expected_result = EosEthTokenDictionary::new(vec![
            get_sample_eos_eth_token_dictionary_entry_2(),
            get_sample_eos_eth_token_dictionary_entry_3(),
        ]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn eos_eth_token_dictionary_should_remove_entry_via_eth_address_and_update_in_db() {
        let token_address = EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let db = get_test_database();
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        dictionary_entries.save_to_db(&db).unwrap();
        dictionary_entries
            .remove_entry_via_eth_address_and_update_in_db(&token_address, &db)
            .unwrap();
        let result = EosEthTokenDictionary::get_from_db(&db).unwrap();
        let expected_result = EosEthTokenDictionary::new(vec![
            get_sample_eos_eth_token_dictionary_entry_2(),
            get_sample_eos_eth_token_dictionary_entry_3(),
        ]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eos_account_name_from_eth_token_address_in_eos_eth_token_dictionary() {
        let eth_address = EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        let expected_result = "sampletoken1".to_string();
        let result = dictionary_entries
            .get_eos_account_name_from_eth_token_address(&eth_address)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_err_when_getting_eos_account_name_from_eth_token_address_if_no_entry_in_dictionary() {
        let eth_address = EthAddress::from_slice(&hex::decode("8f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        let result = dictionary_entries.get_eos_account_name_from_eth_token_address(&eth_address);
        assert!(result.is_err());
    }

    #[test]
    fn should_return_true_if_erc20_token_is_supported() {
        let supported_token_address =
            EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        let result = dictionary_entries.is_token_supported(&supported_token_address);
        assert!(result);
    }

    #[test]
    fn should_return_false_if_erc20_token_is_not_supported() {
        let supported_token_address =
            EthAddress::from_slice(&hex::decode("8f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let dictionary_entries = get_sample_eos_eth_token_dictionary();
        let result = dictionary_entries.is_token_supported(&supported_token_address);
        assert!(!result);
    }

    #[test]
    fn should_complete_eos_eth_token_dictionary_json_bytes_serde_roundtrip() {
        let dictionary_json = get_sample_eos_eth_token_dictionary_json();
        let bytes = dictionary_json.to_bytes().unwrap();
        let result = EosEthTokenDictionaryJson::from_bytes(&bytes).unwrap();
        assert_eq!(result, dictionary_json);
    }

    #[test]
    fn should_complete_dictionary_to_json_roundtrip() {
        let dictionary = get_sample_eos_eth_token_dictionary();
        let json = dictionary.to_json().unwrap();
        let result = EosEthTokenDictionary::from_json(&json).unwrap();
        assert_eq!(result, dictionary);
    }

    #[test]
    fn should_complete_eos_eth_token_dictionary_bytes_serde_roundtrip() {
        let dictionary = get_sample_eos_eth_token_dictionary();
        let bytes = dictionary.to_bytes().unwrap();
        let result = EosEthTokenDictionary::from_bytes(&bytes).unwrap();
        assert_eq!(result, dictionary);
    }

    fn get_sample_dictionary_entry_json_string() -> String {
        "{\"eos_token_decimals\":9,\"eth_token_decimals\":18,\"eos_symbol\":\"SYM\",\"eth_symbol\":\"SYM\",\"eos_address\":\"account_name\",\"eth_address\":\"fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC\"}".to_string()
    }

    #[test]
    fn should_get_dictionary_entry_json_from_str() {
        let json_string = get_sample_dictionary_entry_json_string();
        let result = EosEthTokenDictionaryEntryJson::from_str(&json_string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_dictionary_entry_from_str() {
        let json_string = get_sample_dictionary_entry_json_string();
        let result = EosEthTokenDictionaryEntry::from_str(&json_string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_eos_asset_to_eth_amount() {
        let entry = get_sample_eos_eth_token_dictionary_entry_1();
        let expected_results = vec![
            U256::from_dec_str("1234567891000000000").unwrap(),
            U256::from_dec_str("123456789000000000").unwrap(),
            U256::from_dec_str("12345678000000000").unwrap(),
            U256::from_dec_str("1234567000000000").unwrap(),
            U256::from_dec_str("123456000000000").unwrap(),
            U256::from_dec_str("12345000000000").unwrap(),
            U256::from_dec_str("1234000000000").unwrap(),
            U256::from_dec_str("123000000000").unwrap(),
            U256::from_dec_str("12000000000").unwrap(),
            U256::from_dec_str("1000000000").unwrap(),
            U256::from_dec_str("0").unwrap(),
        ];
        vec![
            "1.234567891 SAM1".to_string(),
            "0.123456789 SAM1".to_string(),
            "0.012345678 SAM1".to_string(),
            "0.001234567 SAM1".to_string(),
            "0.000123456 SAM1".to_string(),
            "0.000012345 SAM1".to_string(),
            "0.000001234 SAM1".to_string(),
            "0.000000123 SAM1".to_string(),
            "0.000000012 SAM1".to_string(),
            "0.000000001 SAM1".to_string(),
            "0.000000000 SAM1".to_string(),
        ]
        .iter()
        .map(|eos_asset| entry.convert_eos_asset_to_eth_amount(eos_asset).unwrap())
        .zip(expected_results.iter())
        .for_each(|(result, expected_result)| assert_eq!(&result, expected_result));
    }

    #[test]
    fn should_convert_eth_amount_to_eos_asset() {
        let entry = get_sample_eos_eth_token_dictionary_entry_1();
        let expected_results = vec![
            "1.234567891 SAM1".to_string(),
            "0.123456789 SAM1".to_string(),
            "0.012345678 SAM1".to_string(),
            "0.001234567 SAM1".to_string(),
            "0.000123456 SAM1".to_string(),
            "0.000012345 SAM1".to_string(),
            "0.000001234 SAM1".to_string(),
            "0.000000123 SAM1".to_string(),
            "0.000000012 SAM1".to_string(),
            "0.000000001 SAM1".to_string(),
            "0.000000000 SAM1".to_string(),
        ];
        vec![
            U256::from_dec_str("1234567891234567891").unwrap(),
            U256::from_dec_str("123456789123456789").unwrap(),
            U256::from_dec_str("12345678912345678").unwrap(),
            U256::from_dec_str("1234567891234567").unwrap(),
            U256::from_dec_str("123456789123456").unwrap(),
            U256::from_dec_str("12345678912345").unwrap(),
            U256::from_dec_str("1234567891234").unwrap(),
            U256::from_dec_str("123456789123").unwrap(),
            U256::from_dec_str("12345678912").unwrap(),
            U256::from_dec_str("1234567891").unwrap(),
            U256::from_dec_str("123456789").unwrap(),
        ]
        .iter()
        .map(|eth_amount| entry.convert_u256_to_eos_asset_string(eth_amount).unwrap())
        .zip(expected_results.iter())
        .for_each(|(result, expected_result)| assert_eq!(&result, expected_result));
    }

    #[test]
    fn should_convert_u64_to_eos_asset() {
        let entry = get_sample_eos_eth_token_dictionary_entry_1();
        let expected_results = vec![
            "123456789.123456789 SAM1".to_string(),
            "12345678.912345678 SAM1".to_string(),
            "1234567.891234567 SAM1".to_string(),
            "123456.789123456 SAM1".to_string(),
            "12345.678912345 SAM1".to_string(),
            "1234.567891234 SAM1".to_string(),
            "123.456789123 SAM1".to_string(),
            "12.345678912 SAM1".to_string(),
            "1.234567891 SAM1".to_string(),
            "0.123456789 SAM1".to_string(),
            "0.012345678 SAM1".to_string(),
            "0.001234567 SAM1".to_string(),
            "0.000123456 SAM1".to_string(),
            "0.000012345 SAM1".to_string(),
            "0.000001234 SAM1".to_string(),
            "0.000000123 SAM1".to_string(),
            "0.000000012 SAM1".to_string(),
            "0.000000001 SAM1".to_string(),
            "0.000000000 SAM1".to_string(),
        ];
        vec![
            123456789123456789_u64,
            12345678912345678_u64,
            1234567891234567_u64,
            123456789123456_u64,
            12345678912345_u64,
            1234567891234_u64,
            123456789123_u64,
            12345678912_u64,
            1234567891_u64,
            123456789_u64,
            12345678_u64,
            1234567_u64,
            123456_u64,
            12345_u64,
            1234_u64,
            123_u64,
            12_u64,
            1_u64,
            0_u64,
        ]
        .iter()
        .map(|u_64| entry.convert_u64_to_eos_asset(*u_64))
        .zip(expected_results.iter())
        .for_each(|(result, expected_result)| assert_eq!(&result, expected_result));
    }

    #[test]
    fn should_get_entry_via_eth_address() {
        let dictionary = get_sample_eos_eth_token_dictionary();
        let expected_result = get_sample_eos_eth_token_dictionary_entry_2();
        let eth_address = EthAddress::from_slice(&hex::decode("9e57cb2a4f462a5258a49e88b4331068a391de66").unwrap());
        let result = dictionary.get_entry_via_eth_address(&eth_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_entry_via_eos_address() {
        let dictionary = get_sample_eos_eth_token_dictionary();
        let expected_result = get_sample_eos_eth_token_dictionary_entry_2();
        let eos_address = EosAccountName::from_str("sampletokens").unwrap();
        let result = dictionary.get_entry_via_eos_address(&eos_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_zero_eos_asset() {
        let entry = EosEthTokenDictionaryEntry::from_str(
            "{\"eth_token_decimals\":18,\"eos_token_decimals\":4,\"eth_symbol\":\"TOK\",\"eos_symbol\":\"EOS\",\"eth_address\":\"9a74c1e17b31745199b229b5c05b61081465b329\",\"eos_address\":\"eosio.token\"}"
        ).unwrap();
        let result = entry.get_zero_eos_asset();
        let expected_result = "0.0000 EOS";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_zero_eos_asset_via_eth_address() {
        let eth_address_str = "9a74c1e17b31745199b229b5c05b61081465b329";
        let eth_address = EthAddress::from_slice(&hex::decode(eth_address_str).unwrap());
        let dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_str(
            &format!("{{\"eth_token_decimals\":18,\"eos_token_decimals\":4,\"eth_symbol\":\"TOK\",\"eos_symbol\":\"EOS\",\"eth_address\":\"{}\",\"eos_address\":\"eosio.token\"}}", eth_address_str)
        ).unwrap()]);
        let result = dictionary
            .get_zero_eos_asset_amount_via_eth_token_address(&eth_address)
            .unwrap();
        let expected_result = "0.0000 EOS";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_dictionary_from_str() {
        let s = "[{\"eth_token_decimals\":18,\"eos_token_decimals\":4,\"eth_symbol\":\"TLOS\",\"eos_symbol\":\"TLOS\",\"eth_address\":\"7825e833d495f3d1c28872415a4aee339d26ac88\",\"eos_address\":\"eosio.token\"},{\"eth_token_decimals\":18,\"eos_token_decimals\":4,\"eth_symbol\":\"pSEEDS\",\"eos_symbol\":\"SEEDS\",\"eth_address\":\"6db338e6ed75f67cd5a4ef8bdf59163b32d4bd46\",\"eos_address\":\"token.seeds\"}]";
        let result = EosEthTokenDictionary::from_str(s);
        assert!(result.is_ok());
    }

    #[test]
    fn should_change_eth_fee_basis_points_in_eos_eth_token_dictionary_and_save_in_db() {
        let db = get_test_database();
        let eth_address = EthAddress::from_slice(&hex::decode("9f57cb2a4f462a5258a49e88b4331068a391de66").unwrap());
        let dictionary = get_sample_eos_eth_token_dictionary();
        let entry = dictionary.get_entry_via_eth_address(&eth_address).unwrap();
        assert_eq!(entry.eth_fee_basis_points, 0);
        let basis_points = 25;
        dictionary
            .change_eth_fee_basis_points_and_update_in_db(&db, &eth_address, basis_points)
            .unwrap();
        let dictionary_from_db = EosEthTokenDictionary::get_from_db(&db).unwrap();
        let result = dictionary_from_db.get_entry_via_eth_address(&eth_address).unwrap();
        assert_eq!(result.eth_fee_basis_points, basis_points);
    }

    #[test]
    fn should_change_eos_fee_basis_points_in_eos_eth_token_dictionary_and_save_in_db() {
        let db = get_test_database();
        let eos_address = EosAccountName::from_str("sampletokens").unwrap();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let entry = dictionary.get_entry_via_eos_address(&eos_address).unwrap();
        let basis_points_before = entry.eos_fee_basis_points;
        let new_basis_points = basis_points_before + 1;
        dictionary
            .change_eos_fee_basis_points_and_update_in_db(&db, &eos_address, new_basis_points)
            .unwrap();
        let dictionary_from_db = EosEthTokenDictionary::get_from_db(&db).unwrap();
        let result = dictionary_from_db
            .get_entry_via_eos_address(&eos_address)
            .unwrap()
            .eos_fee_basis_points;
        assert_eq!(result, new_basis_points);
    }

    #[test]
    fn should_increment_accrued_fees_and_save_in_db() {
        let db = get_test_database();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let eth_address_1 = EthAddress::from_slice(&hex::decode("9f57cb2a4f462a5258a49e88b4331068a391de66").unwrap());
        let eth_address_2 = EthAddress::from_slice(&hex::decode("9e57cb2a4f462a5258a49e88b4331068a391de66").unwrap());
        let expected_fee_1 = U256::from(1337);
        let expected_fee_2 = U256::from(666);
        let fee_tuples = vec![(eth_address_1, expected_fee_1), (eth_address_2, expected_fee_2)];
        let entry_1_before = dictionary.get_entry_via_eth_address(&eth_address_1).unwrap();
        let entry_2_before = dictionary.get_entry_via_eth_address(&eth_address_2).unwrap();
        assert_eq!(entry_1_before.accrued_fees, U256::zero());
        assert_eq!(entry_2_before.accrued_fees, U256::zero());
        dictionary
            .increment_accrued_fees_and_save_in_db(&db, &fee_tuples)
            .unwrap();
        let dictionary_from_db = EosEthTokenDictionary::get_from_db(&db).unwrap();
        let entry_1_after = dictionary_from_db.get_entry_via_eth_address(&eth_address_1).unwrap();
        let entry_2_after = dictionary_from_db.get_entry_via_eth_address(&eth_address_2).unwrap();
        assert_eq!(entry_1_after.accrued_fees, expected_fee_1);
        assert_eq!(entry_2_after.accrued_fees, expected_fee_2);
    }

    #[test]
    fn should_withdraw_fees_from_eos_eth_token_dictionary() {
        let db = get_test_database();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let eth_address = EthAddress::from_slice(&hex::decode("9f57cb2a4f462a5258a49e88b4331068a391de66").unwrap());
        let expected_fee = U256::from(1337);
        let fee_tuples = vec![(eth_address, expected_fee)];
        let entry_1_before = dictionary.get_entry_via_eth_address(&eth_address).unwrap();
        assert_eq!(entry_1_before.accrued_fees, U256::zero());
        dictionary
            .increment_accrued_fees_and_save_in_db(&db, &fee_tuples)
            .unwrap();
        let dictionary_from_db = EosEthTokenDictionary::get_from_db(&db).unwrap();
        let entry_1_after = dictionary_from_db.get_entry_via_eth_address(&eth_address).unwrap();
        assert_eq!(entry_1_after.accrued_fees, expected_fee);
        let result = dictionary_from_db
            .withdraw_fees_and_save_in_db(&db, &eth_address)
            .unwrap();
        let final_dictionary = EosEthTokenDictionary::get_from_db(&db).unwrap();
        let final_entry = final_dictionary.get_entry_via_eth_address(&eth_address).unwrap();
        assert_eq!(final_entry.accrued_fees, U256::zero());
        assert_ne!(final_entry.last_withdrawal, 0);
        assert_eq!(result.0, eth_address);
        assert_eq!(result.1, expected_fee);
    }

    #[test]
    fn should_set_accrued_fees_and_save_in_db() {
        let db = get_test_database();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let eth_address = EthAddress::from_slice(&hex::decode("9f57cb2a4f462a5258a49e88b4331068a391de66").unwrap());
        let expected_fee = U256::from(1337);
        let entry_1_before = dictionary.get_entry_via_eth_address(&eth_address).unwrap();
        assert_eq!(entry_1_before.accrued_fees, U256::zero());
        dictionary
            .set_accrued_fees_and_save_in_db(&db, &eth_address, expected_fee)
            .unwrap();
        let dictionary_from_db = EosEthTokenDictionary::get_from_db(&db).unwrap();
        let result = dictionary_from_db.get_entry_via_eth_address(&eth_address).unwrap();
        assert_eq!(result.accrued_fees, expected_fee);
    }

    #[test]
    fn should_get_entry_via_eos_address_and_symbol() {
        let dictionary = get_sample_eos_eth_token_dictionary();
        let symbol = "SAM2";
        let address = "sampletokens";
        let result = dictionary
            .get_entry_via_eos_address_and_symbol(symbol, address)
            .unwrap();
        let expected_result = get_sample_eos_eth_token_dictionary_entry_2();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_error_when_getting_entry_via_eos_address_and_symbol_if_not_extant() {
        let dictionary = get_sample_eos_eth_token_dictionary();
        let symbol = "SAM2";
        let address = "wrongnameaaa";
        let expected_error = format!("No entry with EOS symbol {} & address {}!", symbol, address);
        match dictionary.get_entry_via_eos_address_and_symbol(symbol, address) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_error_when_getting_entry_via_eos_address_and_symbol_if_more_than_one_entry() {
        let dictionary = EosEthTokenDictionary::new(vec![
            get_sample_eos_eth_token_dictionary_entry_1(),
            get_sample_eos_eth_token_dictionary_entry_2(),
            get_sample_eos_eth_token_dictionary_entry_3(),
            get_sample_eos_eth_token_dictionary_entry_2(),
        ]);
        let symbol = "SAM2";
        let address = "sampletokens";
        let expected_error = format!("Found > 1 entries with EOS symbol {} & address {}!", symbol, address);
        match dictionary.get_entry_via_eos_address_and_symbol(symbol, address) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_convert_dictionary_to_unique_eos_accounts() {
        let dictionary = EosEthTokenDictionary::new(vec![
            get_sample_eos_eth_token_dictionary_entry_1(),
            get_sample_eos_eth_token_dictionary_entry_2(),
            get_sample_eos_eth_token_dictionary_entry_3(),
            get_sample_eos_eth_token_dictionary_entry_2(),
        ]);
        let expected_result = vec![
            EosAccountName::from_str("sampletoken1").unwrap(),
            EosAccountName::from_str("sampletokens").unwrap(),
            EosAccountName::from_str("testpethxxxx").unwrap(),
        ];
        let result = dictionary.to_unique_eos_accounts().unwrap();
        assert_eq!(result, expected_result);
        assert_eq!(result.len(), expected_result.len());
    }

    #[test]
    fn should_fail_to_convert_eos_amount_if_wrong_number_of_decimals() {
        let entry = get_sample_eos_eth_token_dictionary_entry_1();
        let wrong_decimals_eos_asset = "0.0000000010000000000 SAM1".to_string();
        let (_, fraction_str) =
            EosEthTokenDictionaryEntry::get_decimal_and_fractional_parts_of_eos_asset(&wrong_decimals_eos_asset);
        let num_decimals = fraction_str.len();
        let expected_num_decimals = entry.eos_token_decimals;
        assert_ne!(num_decimals, entry.eos_token_decimals);
        let expected_error = format!(
            "Expected {} decimals in EOS asset, found {}! ",
            expected_num_decimals, num_decimals
        );
        match entry.convert_eos_asset_to_eth_amount(&wrong_decimals_eos_asset) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received"),
        }
    }
}
