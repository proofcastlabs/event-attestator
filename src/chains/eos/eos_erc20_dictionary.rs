use ethereum_types::Address as EthAddress;
use derive_more::{
    Deref,
    DerefMut,
    Constructor,
};
use crate::{
    traits::DatabaseInterface,
    chains::eos::eos_state::EosState,
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    types::{
        Byte,
        Bytes,
        Result,
    },
    chains::eos::eos_constants::EOS_ERC20_DICTIONARY,
};


#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, DerefMut)]
pub struct EosErc20Dictionary(pub Vec<EosErc20DictionaryEntry>);

impl EosErc20Dictionary {
    fn to_hex_strings(&self) -> Result<Vec<String>> {
        self
            .iter()
            .map(|eos_erc20_dictionary| eos_erc20_dictionary.to_bytes())
            .map(|bytes: Result<Bytes>| -> Result<String> { Ok(hex::encode(&bytes?)) })
            .collect()
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

    fn from_bytes(bytes: &Bytes) -> Result<Self> {
        EosErc20DictionaryJson::from_bytes(bytes).and_then(|json| Self::from_json(&json))
    }

    fn add(mut self, eos_erc20_dictionary: EosErc20DictionaryEntry) -> Result<Self> {
        match self.contains(&eos_erc20_dictionary) {
            true => {
                debug!("Not adding new `EosErc20DictionaryEntry` ∵ account name already extant!");
                Ok(self)
            }
            false => {
                self.push(eos_erc20_dictionary);
                Ok(self)
            }
        }
    }

    fn remove(mut self, eos_erc20_dictionary: &EosErc20DictionaryEntry) -> Result<Self> {
        match self.contains(&eos_erc20_dictionary) {
            false => Ok(self),
            true => {
                debug!("Removing `EosErc20DictionaryEntry`: {:?}", eos_erc20_dictionary);
                self.retain(|x| x != eos_erc20_dictionary);
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

    fn add_and_update_in_db<D>(
        self,
        eos_erc20_dictionary: EosErc20DictionaryEntry,
        db: &D
    ) -> Result<Self> where D: DatabaseInterface {
        self.add(eos_erc20_dictionary).and_then(|new_self| { new_self.save_to_db(db)?; Ok(new_self) })

    }

    fn remove_and_update_in_db<D>(
        self,
        eos_erc20_dictionary: &EosErc20DictionaryEntry,
        db: &D
    ) -> Result<Self> where D: DatabaseInterface {
        match self.contains(eos_erc20_dictionary) {
            true => self.remove(eos_erc20_dictionary).and_then(|new_self| { new_self.save_to_db(db)?; Ok(new_self) }),
            false => Ok(self)
        }
    }

    pub fn get_eos_account_name_from_eth_token_address(&self, eth_erc20_token_address: &EthAddress) -> Result<String> {
        for entry in self.iter() {
            if &entry.eth_erc20_token_address == eth_erc20_token_address {
                return Ok(entry.token_eos_account_name.to_string())
            }
        }
        Err(format!("No `EosErc20DictionaryEntry` exists with address: {}", eth_erc20_token_address).into())
    }

    pub fn is_token_supported(&self, eth_erc20_token_address: &EthAddress) -> bool {
        self.get_eos_account_name_from_eth_token_address(eth_erc20_token_address).is_ok()
    }

    pub fn to_eth_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|entry| entry.eth_erc20_token_address).collect()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosErc20DictionaryJson(pub Vec<EosErc20DictionaryEntryJson>);

impl EosErc20DictionaryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &Bytes) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EosErc20DictionaryEntry {
    token_eos_account_name: String,
    eth_erc20_token_address: EthAddress,
}

impl EosErc20DictionaryEntry {
    fn to_json(&self) -> EosErc20DictionaryEntryJson {
        EosErc20DictionaryEntryJson {
            eth_erc20_token_address: hex::encode(self.eth_erc20_token_address),
            token_eos_account_name: self.token_eos_account_name.to_string(),
        }
    }

    pub fn from_json(json: &EosErc20DictionaryEntryJson) -> Result<Self> {
        Ok(Self {
            token_eos_account_name: json.token_eos_account_name.to_string(),
            eth_erc20_token_address: EthAddress::from_slice(&hex::decode(&json.eth_erc20_token_address)?),
        })
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_json(&serde_json::from_slice(bytes)?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EosErc20DictionaryEntryJson {
    token_eos_account_name: String,
    eth_erc20_token_address: String,
}

impl EosErc20DictionaryEntryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }
}

pub fn get_erc20_dictionary_from_db_and_add_to_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Getting `EosERc20Dictionary` and adding to state...");
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
        let eth_erc20_token_address = EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        let expected_result = "SampleToken_1".to_string();
        let result = dictionary_entries.get_eos_account_name_from_eth_token_address(&eth_erc20_token_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_err_when_getting_eos_account_name_from_eth_token_address_if_no_entry_in_dictionary() {
        let eth_erc20_token_address = EthAddress::from_slice(&hex::decode("8f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let dictionary_entries = get_sample_eos_erc20_dictionary();
        let result = dictionary_entries.get_eos_account_name_from_eth_token_address(&eth_erc20_token_address);
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
}
