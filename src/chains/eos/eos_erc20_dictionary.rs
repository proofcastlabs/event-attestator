use std::str::FromStr;
use serde_json::Value as JsonValue;
use ethereum_types::{
    U256 as EthAmount,
    Address as EthAddress,
};
use eos_primitives::AccountName as EosAccountName;
use derive_more::{
    Deref,
    DerefMut,
    Constructor,
};
use crate::{
    traits::DatabaseInterface,
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    utils::{
        truncate_str,
        left_pad_with_zeroes,
        split_string_at_index,
        right_pad_with_zeroes,
        right_pad_or_truncate,
    },
    types::{
        Byte,
        Bytes,
        Result,
    },
    chains::{
        eth::eth_state::EthState,
        eos::{
            eos_state::EosState,
            eos_constants::EOS_ERC20_DICTIONARY,
        },
    },
};


#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, DerefMut)]
pub struct EosErc20Dictionary(pub Vec<EosErc20DictionaryEntry>);

impl EosErc20Dictionary {
    pub fn from_str(json_string: &str) -> Result<Self> {
        Self::from_json(&EosErc20DictionaryJson::from_str(json_string)?)
    }

    pub fn to_json(&self) -> Result<EosErc20DictionaryJson> {
        Ok(EosErc20DictionaryJson::new(self.iter().map(|entry| entry.to_json()).collect()))
    }

    pub fn from_json(json: &EosErc20DictionaryJson) -> Result<Self> {
        Ok(Self(
            json
                .iter()
                .map(|entry_json| EosErc20DictionaryEntry::from_json(&entry_json))
                .collect::<Result<Vec<EosErc20DictionaryEntry>>>()?
        ))
    }

    fn to_bytes(&self) -> Result<Bytes> {
        self.to_json()?.to_bytes()
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        EosErc20DictionaryJson::from_bytes(bytes).and_then(|json| Self::from_json(&json))
    }

    fn add(mut self, entry: EosErc20DictionaryEntry) -> Result<Self> {
        info!("✔ Adding `EosErc20Dictionary` entry: {:?}...", entry);
        match self.contains(&entry) {
            true => {
                info!("Not adding new `EosErc20DictionaryEntry` ∵ account name already extant!");
                Ok(self)
            }
            false => {
                self.push(entry);
                Ok(self)
            }
        }
    }

    fn remove(mut self, entry: &EosErc20DictionaryEntry) -> Result<Self> {
        info!("✔ Removing `EosErc20Dictionary` entry: {:?}...", entry);
        match self.contains(&entry) {
            false => Ok(self),
            true => {
                info!("Removing `EosErc20DictionaryEntry`: {:?}", entry);
                self.retain(|x| x != entry);
                Ok(self)
            }
        }
    }

    pub fn save_to_db<D>(&self, db: &D) -> Result<()> where D: DatabaseInterface {
        db.put(EOS_ERC20_DICTIONARY.to_vec(), self.to_bytes()?, MIN_DATA_SENSITIVITY_LEVEL)
    }

    pub fn get_from_db<D>(db: &D) -> Result<Self> where D: DatabaseInterface {
        info!("✔ Getting `EosErc20DictionaryJson` from db...");
        match db.get(EOS_ERC20_DICTIONARY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(_) => {
                info!("✔ No `EosErc20DictionaryJson` in db! Initializing a new one...");
                Ok(Self::new(vec![]))
            }
        }
    }

    pub fn add_and_update_in_db<D>(
        self,
        entry: EosErc20DictionaryEntry,
        db: &D
    ) -> Result<Self> where D: DatabaseInterface {
        self.add(entry).and_then(|new_self| { new_self.save_to_db(db)?; Ok(new_self) })
    }

    pub fn remove_and_update_in_db<D>(
        self,
        entry: &EosErc20DictionaryEntry,
        db: &D
    ) -> Result<Self> where D: DatabaseInterface {
        match self.contains(entry) {
            true => self.remove(entry).and_then(|new_self| { new_self.save_to_db(db)?; Ok(new_self) }),
            false => Ok(self)
        }
    }

    pub fn get_entry_via_eth_token_address(&self, address: &EthAddress) -> Result<EosErc20DictionaryEntry> {
        for entry in self.iter() { if &entry.eth_address == address { return Ok(entry.clone()) } };
        Err(format!("No `EosErc20DictionaryEntry` exists with address: {}", address).into())
    }

    pub fn get_eos_account_name_from_eth_token_address(&self, address: &EthAddress) -> Result<String> {
        self.get_entry_via_eth_token_address(address)
            .map(|entry| entry.eos_address.to_string())
    }

    pub fn is_token_supported(&self, address: &EthAddress) -> bool {
        self.get_eos_account_name_from_eth_token_address(address).is_ok()
    }

    pub fn to_eth_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|entry| entry.eth_address).collect()
    }

    pub fn to_eos_accounts(&self) -> Result<Vec<EosAccountName>> {
        self.iter().map(|entry| Ok(EosAccountName::from_str(&entry.eos_address)?)).collect()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosErc20DictionaryJson(pub Vec<EosErc20DictionaryEntryJson>);

impl EosErc20DictionaryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn from_str(json_string: &str) -> Result<Self> {
        let intermediary: Vec<JsonValue> = serde_json::from_str(json_string)?;
        Ok(Self::new(
            intermediary
                .iter()
                .map(|json_value| json_value.to_string())
                .map(|entry_json_string| EosErc20DictionaryEntryJson::from_str(&entry_json_string))
                .collect::<Result<Vec<EosErc20DictionaryEntryJson>>>()?
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EosErc20DictionaryEntry {
    eth_token_decimals: usize,
    eos_token_decimals: usize,
    eos_symbol: String,
    eth_symbol: String,
    eos_address: String,
    eth_address: EthAddress,
}

impl EosErc20DictionaryEntry {
    fn to_json(&self) -> EosErc20DictionaryEntryJson {
        EosErc20DictionaryEntryJson {
            eth_token_decimals: self.eth_token_decimals,
            eos_token_decimals: self.eos_token_decimals,
            eos_symbol: self.eos_symbol.to_string(),
            eth_symbol: self.eth_symbol.to_string(),
            eos_address: self.eos_address.to_string(),
            eth_address: hex::encode(self.eth_address),
        }
    }

    pub fn from_json(json: &EosErc20DictionaryEntryJson) -> Result<Self> {
        Ok(Self {
            eth_token_decimals: json.eth_token_decimals,
            eos_token_decimals: json.eos_token_decimals,
            eos_symbol: json.eos_symbol.to_string(),
            eth_symbol: json.eth_symbol.to_string(),
            eos_address: json.eos_address.to_string(),
            eth_address: EthAddress::from_slice(&hex::decode(&json.eth_address)?),
        })
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_json(&serde_json::from_slice(bytes)?)
    }

    pub fn from_str(json_string: &str) -> Result<Self> {
        EosErc20DictionaryEntryJson::from_str(json_string).and_then(|entry_json| Self::from_json(&entry_json))
    }


    fn remove_symbol_from_eos_asset(eos_asset: &str) -> &str {
        eos_asset.split_whitespace().collect::<Vec<&str>>()[0]
    }

    fn get_decimal_and_fractional_parts_of_eos_asset(eos_asset: &str) -> (&str, &str) {
        let parts = Self::remove_symbol_from_eos_asset(eos_asset).split(".").collect::<Vec<&str>>();
        let decimal_part = parts[0];
        let fractional_part = if parts.len() > 1 { parts[1] } else { "" };
        (decimal_part, fractional_part)
    }

    pub fn convert_eos_asset_to_eth_amount(&self, eos_asset: &str) -> Result<EthAmount> {
        let (decimal_str, fraction_str) = Self::get_decimal_and_fractional_parts_of_eos_asset(eos_asset);
        let augmented_fraction_str = if self.eth_token_decimals > self.eos_token_decimals {
            right_pad_with_zeroes(fraction_str, self.eth_token_decimals)
        } else if self.eth_token_decimals == self.eos_token_decimals {
            fraction_str.to_string()
        } else {
            truncate_str(fraction_str, self.eos_token_decimals - self.eth_token_decimals).to_string()
        };
        Ok(EthAmount::from_dec_str(&format!("{}{}", decimal_str, augmented_fraction_str))?)
    }

    pub fn convert_u256_to_eos_asset_string(&self, amount: &EthAmount) -> Result<String> {
        let amount_str = amount.to_string();
        if amount_str.len() >= self.eth_token_decimals {
            let decimal_point_index = amount_str.len() - self.eth_token_decimals;
            let (decimal_str, fraction_str) = split_string_at_index(&amount_str, decimal_point_index);
            let augmented_fraction_str = right_pad_or_truncate(&fraction_str, self.eos_token_decimals);
            let augmented_decimal_str = if decimal_str == "" { "0".to_string() } else { decimal_str };
            Ok(format!("{}.{} {}", augmented_decimal_str, augmented_fraction_str, self.eos_symbol.to_uppercase()))
        } else {
            let fraction_str = left_pad_with_zeroes(&amount_str, self.eth_token_decimals);
            let augmented_fraction_str = right_pad_or_truncate(&fraction_str, self.eos_token_decimals);
            Ok(format!("0.{} {}", augmented_fraction_str, self.eos_symbol.to_uppercase()))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EosErc20DictionaryEntryJson {
    eth_token_decimals: usize,
    eos_token_decimals: usize,
    eth_symbol: String,
    eos_symbol: String,
    eth_address: String,
    eos_address: String,
}

impl EosErc20DictionaryEntryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_str(json_string: &str) -> Result<Self> {
        match serde_json::from_str(json_string) {
            Ok(result) => Ok(result),
            Err(err) => Err(err.into())
        }
    }
}

pub fn get_erc20_dictionary_from_db_and_add_to_eos_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Getting `EosERc20Dictionary` and adding to EOS state...");
    EosErc20Dictionary::get_from_db(&state.db).and_then(|dictionary| state.add_eos_erc20_dictionary(dictionary))
}

pub fn get_erc20_dictionary_from_db_and_add_to_eth_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Getting `EosERc20Dictionary` and adding to ETH state...");
    EosErc20Dictionary::get_from_db(&state.db).and_then(|dictionary| state.add_eos_erc20_dictionary(dictionary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_utils::get_test_database,
        chains::eos::eos_test_utils::{
            get_sample_eos_erc20_dictionary,
            get_sample_eos_erc20_dictionary_json,
            get_sample_eos_erc20_dictionary_entry_1,
            get_sample_eos_erc20_dictionary_entry_2,
        },
    };


    #[test]
    fn eos_erc20_dictionary_should_contain_eos_erc20_dictionary_entry() {
        let dictionary_entry = get_sample_eos_erc20_dictionary_entry_1();
        let dictionary = get_sample_eos_erc20_dictionary();
        assert!(dictionary.contains(&dictionary_entry));
    }

    #[test]
    fn eos_erc20_dictionary_should_no_contain_other_eos_erc20_dictionary_entry() {
        let token_address_hex = "9e57CB2a4F462a5258a49E88B4331068a391DE66".to_string();
        let other_dictionary_entry = EosErc20DictionaryEntry::new(
            18,
            9,
            "SYM".to_string(),
            "SYM".to_string(),
            "SampleTokenx".to_string(),
            EthAddress::from_slice(&hex::decode(&token_address_hex).unwrap()),
        );
        let dictionary = get_sample_eos_erc20_dictionary();
        assert!(!dictionary.contains(&other_dictionary_entry));
    }

    #[test]
    fn should_push_into_eos_erc20_dictionary_if_entry_not_extant() {
        let expected_num_entries_before = 1;
        let expected_num_entries_after = 2;
        let dictionary_entries = EosErc20Dictionary::new(vec![get_sample_eos_erc20_dictionary_entry_1()]);
        assert_eq!(dictionary_entries.len(), expected_num_entries_before);
        let updated_dictionary = dictionary_entries.add(get_sample_eos_erc20_dictionary_entry_2()).unwrap();
        assert_eq!(updated_dictionary.len(), expected_num_entries_after);
    }

    #[test]
    fn should_not_push_into_eos_erc20_dictionary_if_entry_extant() {
        let expected_num_account_names = 2;
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        assert_eq!(dictionary_entries.len(), expected_num_account_names);
        let updated_dictionary = dictionary_entries.add(get_sample_eos_erc20_dictionary_entry_1()).unwrap();
        assert_eq!(updated_dictionary.len(), expected_num_account_names);

    }

    #[test]
    fn should_remove_entry_from_eos_erc20_dictionary() {
        let expected_num_entries_before = 2;
        let expected_num_entries_after = 1;
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        assert_eq!(dictionary_entries.len(), expected_num_entries_before);
        let updated_dictionary = dictionary_entries.remove(&get_sample_eos_erc20_dictionary_entry_2()).unwrap();
        assert_eq!(updated_dictionary.len(), expected_num_entries_after);
    }

    #[test]
    fn should_savee_eos_erc20_dictionary_in_db() {
        let db = get_test_database();
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        dictionary_entries.save_to_db(&db).unwrap();
        let result = db.get(EOS_ERC20_DICTIONARY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL).unwrap();
        assert_eq!(result, dictionary_entries.to_bytes().unwrap());
    }

    #[test]
    fn get_from_db_should_get_empty_eos_erc20_dictionary_if_non_extant() {
        let db = get_test_database();
        let expected_result = EosErc20Dictionary::new(vec![]);
        let result = EosErc20Dictionary::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_from_db_should_get_correct_eos_erc20_dictionary_if_extant() {
        let db = get_test_database();
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        dictionary_entries.save_to_db(&db).unwrap();
        let result = EosErc20Dictionary::get_from_db(&db).unwrap();
        assert_eq!(result, dictionary_entries);
    }

    #[test]
    fn eos_erc20_dictionary_should_add_new_entry_and_update_in_db() {
        let db = get_test_database();
        let dictionary_entries = EosErc20Dictionary::new(vec![get_sample_eos_erc20_dictionary_entry_1()]);
        dictionary_entries.add_and_update_in_db(get_sample_eos_erc20_dictionary_entry_2(), &db).unwrap();
        let result = EosErc20Dictionary::get_from_db(&db).unwrap();
        assert_eq!(result, get_sample_eos_erc20_dictionary());
    }

    #[test]
    fn eos_erc20_dictionary_should_remove_entry_and_update_in_db() {
        let db = get_test_database();
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        dictionary_entries.save_to_db(&db).unwrap();
        dictionary_entries.remove_and_update_in_db(&get_sample_eos_erc20_dictionary_entry_1(), &db).unwrap();
        let result = EosErc20Dictionary::get_from_db(&db).unwrap();
        let expected_result = EosErc20Dictionary::new(vec![get_sample_eos_erc20_dictionary_entry_2()]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eos_account_name_from_eth_token_address_in_eos_erc20_dictionary() {
        let eth_address = EthAddress::from_slice(
            &hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()
        );
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        let expected_result = "SampleToken_1".to_string();
        let result = dictionary_entries.get_eos_account_name_from_eth_token_address(&eth_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_err_when_getting_eos_account_name_from_eth_token_address_if_no_entry_in_dictionary() {
        let eth_address = EthAddress::from_slice(
            &hex::decode("8f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()
        );
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        let result = dictionary_entries.get_eos_account_name_from_eth_token_address(&eth_address);
        assert!(result.is_err());
    }

    #[test]
    fn should_return_true_if_erc20_token_is_supported() {
        let supported_token_address = EthAddress::from_slice(
            &hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()
        );
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        let result = dictionary_entries.is_token_supported(&supported_token_address);
        assert!(result);
    }

    #[test]
    fn should_return_false_if_erc20_token_is_not_supported() {
        let supported_token_address = EthAddress::from_slice(
            &hex::decode("8f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()
        );
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        let result = dictionary_entries.is_token_supported(&supported_token_address);
        assert!(!result);
    }

    #[test]
    fn should_complete_eos_erc20_dictionary_json_bytes_serde_roundtrip() {
        let dictionary_json = get_sample_eos_erc20_dictionary_json();
        let bytes = dictionary_json.to_bytes().unwrap();
        let result = EosErc20DictionaryJson::from_bytes(&bytes).unwrap();
        assert_eq!(result, dictionary_json);
    }

    #[test]
    fn should_complete_dictionary_to_json_roundtrip() {
        let dictionary = get_sample_eos_erc20_dictionary();
        let json = dictionary.to_json().unwrap();
        let result = EosErc20Dictionary::from_json(&json).unwrap();
        assert_eq!(result, dictionary);
    }

    #[test]
    fn should_complete_eos_erc20_dictionary_bytes_serde_roundtrip() {
        let dictionary = get_sample_eos_erc20_dictionary();
        let bytes = dictionary.to_bytes().unwrap();
        let result = EosErc20Dictionary::from_bytes(&bytes).unwrap();
        assert_eq!(result, dictionary);

    }

    fn get_sample_dictionary_entry_json_string() -> String {
        "{\"eos_token_decimals\":9,\"eth_token_decimals\":18,\"eos_symbol\":\"SYM\",\"eth_symbol\":\"SYM\",\"eos_address\":\"account_name\",\"eth_address\":\"fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC\"}".to_string()
    }

    fn get_sample_dictionary_json_string() -> String {
        "[{\"eos_token_decimals\":9,\"eth_token_decimals\":18,\"eos_symbol\":\"SYM\",\"eth_symbol\":\"SYM2\",\"eos_address\":\"somename1\",\"eth_address\":\"fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC\"},{\"eos_token_decimals\":9,\"eth_token_decimals\":18,\"eos_symbol\":\"SYM\",\"eth_symbol\":\"SYM2\",\"eos_address\":\"somename2\",\"eth_address\":\"edB86cd455ef3ca43f0e227e00469C3bDFA40628\"}]".to_string()
    }

    #[test]
    fn should_get_dictionary_entry_json_from_str() {
        let json_string = get_sample_dictionary_entry_json_string();
        let result = EosErc20DictionaryEntryJson::from_str(&json_string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_dictionary_entry_from_str() {
        let json_string = get_sample_dictionary_entry_json_string();
        let result = EosErc20DictionaryEntry::from_str(&json_string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_dictionary_json_from_str() {
        let json_string = get_sample_dictionary_json_string();
        let result = EosErc20DictionaryJson::from_str(&json_string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_dictionary_from_str() {
        let json_string = get_sample_dictionary_json_string();
        let result = EosErc20Dictionary::from_str(&json_string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_eos_asset_to_eth_amount() {
        let entry = get_sample_eos_erc20_dictionary_entry_1();
        let expected_results = vec![
            EthAmount::from_dec_str("1234567891000000000").unwrap(),
            EthAmount::from_dec_str("123456789000000000").unwrap(),
            EthAmount::from_dec_str("12345678000000000").unwrap(),
            EthAmount::from_dec_str("1234567000000000").unwrap(),
            EthAmount::from_dec_str("123456000000000").unwrap(),
            EthAmount::from_dec_str("12345000000000").unwrap(),
            EthAmount::from_dec_str("1234000000000").unwrap(),
            EthAmount::from_dec_str("123000000000").unwrap(),
            EthAmount::from_dec_str("12000000000").unwrap(),
            EthAmount::from_dec_str("1000000000").unwrap(),
            EthAmount::from_dec_str("0").unwrap(),
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
            .map(|eos_asset| entry.convert_eos_asset_to_eth_amount(&eos_asset).unwrap())
            .zip(expected_results.iter())
            .map(|(result, expected_result)| assert_eq!(&result, expected_result))
            .for_each(drop);
    }

    #[test]
    fn should_convert_eth_amount_to_eos_asset() {
        let entry = get_sample_eos_erc20_dictionary_entry_1();
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
            EthAmount::from_dec_str("1234567891234567891").unwrap(),
            EthAmount::from_dec_str("123456789123456789").unwrap(),
            EthAmount::from_dec_str("12345678912345678").unwrap(),
            EthAmount::from_dec_str("1234567891234567").unwrap(),
            EthAmount::from_dec_str("123456789123456").unwrap(),
            EthAmount::from_dec_str("12345678912345").unwrap(),
            EthAmount::from_dec_str("1234567891234").unwrap(),
            EthAmount::from_dec_str("123456789123").unwrap(),
            EthAmount::from_dec_str("12345678912").unwrap(),
            EthAmount::from_dec_str("1234567891").unwrap(),
            EthAmount::from_dec_str("123456789").unwrap(),
        ]
            .iter()
            .map(|eth_amount| entry.convert_u256_to_eos_asset_string(&eth_amount).unwrap())
            .zip(expected_results.iter())
            .map(|(result, expected_result)| assert_eq!(&result, expected_result))
            .for_each(drop);
    }
}
