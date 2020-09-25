use ethereum_types::Address as EthAddress;
use derive_more::{
    Deref,
    DerefMut,
    Constructor,
};
use serde_json::{
    json,
    Value as Json,
};
use crate::{
    traits::DatabaseInterface,
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    types::{
        Byte,
        Bytes,
        Result,
    },
    chains::eos::eos_constants::EOS_ERC20_ACCOUNT_NAMES,
};


#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, DerefMut)]
pub struct EosErc20AccountNames(pub Vec<EosErc20AccountName>);

impl EosErc20AccountNames {
    fn to_hex_strings(&self) -> Result<Vec<String>> {
        self
            .iter()
            .map(|eos_erc20_account_name| eos_erc20_account_name.to_bytes())
            .map(|bytes: Result<Bytes>| -> Result<String> { Ok(hex::encode(&bytes?)) })
            .collect()
    }

    fn to_json(&self) -> Result<Json> {
        Ok(json!({ "account_names": self.to_hex_strings()? }))
    }

    fn from_json(json: EosErc20AccountNamesJson) -> Result<Self> {
        Ok(Self(
            json
                .account_names
                .iter()
                .map(|bytes| -> Result<EosErc20AccountName> { EosErc20AccountName::from_bytes(&hex::decode(bytes)?) })
                .collect::<Result<Vec<EosErc20AccountName>>>()?
        ))
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_json(serde_json::from_slice(bytes)?)
    }

    fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.to_json()?)?)
    }

    fn add(mut self, eos_erc20_account_name: EosErc20AccountName) -> Result<Self> {
        match self.contains(&eos_erc20_account_name) {
            true => {
                debug!("Not adding new `EosErc20AccountName` ∵ account name already extant!");
                Ok(self)
            }
            false => {
                self.push(eos_erc20_account_name);
                Ok(self)
            }
        }
    }

    fn remove(mut self, eos_erc20_account_name: &EosErc20AccountName) -> Result<Self> {
        match self.contains(&eos_erc20_account_name) {
            false => Ok(self),
            true => {
                debug!("Removing `EosErc20AccountName`: {:?}", eos_erc20_account_name);
                self.retain(|x| x != eos_erc20_account_name);
                Ok(self)
            }
        }
    }

    fn save_to_db<D>(&self, db: &D) -> Result<()> where D: DatabaseInterface {
        db.put(EOS_ERC20_ACCOUNT_NAMES.to_vec(), self.to_bytes()?, MIN_DATA_SENSITIVITY_LEVEL)
    }

    pub fn get_from_db<D>(db: &D) -> Result<Self> where D: DatabaseInterface {
        info!("✔ Getting `EosErc20AccountNamesJson` from db...");
        match db.get(EOS_ERC20_ACCOUNT_NAMES.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(_) => {
                info!("✔ No `EosErc20AccountNamesJson` in db! Initializing a new one...");
                Ok(Self::new(vec![]))
            }
        }
    }

    fn add_and_update_in_db<D>(
        self,
        eos_erc20_account_name: EosErc20AccountName,
        db: &D
    ) -> Result<Self> where D: DatabaseInterface {
        self.add(eos_erc20_account_name).and_then(|new_self| { new_self.save_to_db(db)?; Ok(new_self) })

    }

    fn remove_and_update_in_db<D>(
        self,
        eos_erc20_account_name: &EosErc20AccountName,
        db: &D
    ) -> Result<Self> where D: DatabaseInterface {
        match self.contains(eos_erc20_account_name) {
            true => self.remove(eos_erc20_account_name).and_then(|new_self| { new_self.save_to_db(db)?; Ok(new_self) }),
            false => Ok(self)
        }
    }

    fn get_account_name_from_token_address(&self, token_address: &EthAddress) -> Result<String> {
        for account_name in self.iter() {
            if &account_name.token_address == token_address {
                return Ok(account_name.token_name.to_string())
            }
        }
        Err(format!("No `EosErc20AccountName` exists with address: {}", token_address).into())
    }

    pub fn is_token_supported(&self, token_address: &EthAddress) -> bool {
        self.get_account_name_from_token_address(token_address).is_ok()
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EosErc20AccountNamesJson {
    account_names: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EosErc20AccountnamesJson(pub Vec<String>);

#[derive(Debug, Clone, Eq, PartialEq, Constructor)]
pub struct EosErc20AccountName {
    token_name: String,
    token_address: EthAddress,
}

impl EosErc20AccountName {
    fn to_json(&self) -> Json {
        json!({ "token_name": self.token_name, "token_address": hex::encode(self.token_address) })
    }

    pub fn from_json(json: &EosErc20AccountNameJson) -> Result<Self> {
        Ok(Self {
            token_name: json.token_name.to_string(),
            token_address: EthAddress::from_slice(&hex::decode(&json.token_address)?),
        })
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.to_json())?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_json(&serde_json::from_slice(bytes)?)
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EosErc20AccountNameJson {
    token_name: String,
    token_address: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_test_database;

    fn get_sample_eos_erc20_account_name_1() -> EosErc20AccountName {
        let token_address_hex = "9f57CB2a4F462a5258a49E88B4331068a391DE66".to_string();
        EosErc20AccountName::new(
            "SampleToken_1".to_string(),
            EthAddress::from_slice(&hex::decode(&token_address_hex).unwrap()),
        )
    }

    fn get_sample_eos_erc20_account_name_2() -> EosErc20AccountName {
        let token_address_hex = "9e57CB2a4F462a5258a49E88B4331068a391DE66".to_string();
        EosErc20AccountName::new(
            "SampleToken_2".to_string(),
            EthAddress::from_slice(&hex::decode(&token_address_hex).unwrap()),
        )
    }

    fn get_sample_eos_erc20_account_names() -> EosErc20AccountNames {
        EosErc20AccountNames::new(vec![
            get_sample_eos_erc20_account_name_1(),
            get_sample_eos_erc20_account_name_2(),
        ])
    }

    #[test]
    fn should_complete_eos_erc20_account_name_serde_round_trip() {
        let eos_erc20_account_name = get_sample_eos_erc20_account_name_1();
        let bytes = eos_erc20_account_name.to_bytes().unwrap();
        let result = EosErc20AccountName::from_bytes(&bytes).unwrap();
        assert_eq!(result, eos_erc20_account_name);
    }

    #[test]
    fn should_complete_eos_erc20_account_names_serde_round_trip() {
        let eos_erc20_account_names = get_sample_eos_erc20_account_names();
        let bytes = eos_erc20_account_names.to_bytes().unwrap();
        let result = EosErc20AccountNames::from_bytes(&bytes).unwrap();
        assert_eq!(result, eos_erc20_account_names);
    }

    #[test]
    fn should_contain_eos_erc20_account_name() {
        let eos_erc20_account_name = get_sample_eos_erc20_account_name_1();
        let eos_erc20_account_names = get_sample_eos_erc20_account_names();
        assert!(eos_erc20_account_names.contains(&eos_erc20_account_name));
    }

    #[test]
    fn should_not_other_contain_eos_erc20_account_name() {
        let token_address_hex = "9e57CB2a4F462a5258a49E88B4331068a391DE66".to_string();
        let other_eos_erc20_account_name = EosErc20AccountName::new(
            "SampleTokenx".to_string(),
            EthAddress::from_slice(&hex::decode(&token_address_hex).unwrap()),
        );
        let eos_erc20_account_names = get_sample_eos_erc20_account_names();
        assert!(!eos_erc20_account_names.contains(&other_eos_erc20_account_name));
    }

    #[test]
    fn should_push_into_eos_erc20_account_names() {
        let expected_num_account_names_before = 1;
        let expected_num_account_names_after = 2;
        let account_names = EosErc20AccountNames::new(vec![get_sample_eos_erc20_account_name_1()]);
        assert_eq!(account_names.len(), expected_num_account_names_before);
        let new_account_names = account_names.add(get_sample_eos_erc20_account_name_2()).unwrap();
        assert_eq!(new_account_names.len(), expected_num_account_names_after);
    }

    #[test]
    fn should_not_push_into_eos_erc20_account_names_if_extant() {
        let expected_num_account_names = 2;
        let account_names = get_sample_eos_erc20_account_names();
        assert_eq!(account_names.len(), expected_num_account_names);
        let new_account_names = account_names.add(get_sample_eos_erc20_account_name_1()).unwrap();
        assert_eq!(new_account_names.len(), expected_num_account_names);

    }

    #[test]
    fn should_remove_account_name() {
        let expected_num_account_names_before = 2;
        let expected_num_account_names_after = 1;
        let account_names = get_sample_eos_erc20_account_names();
        assert_eq!(account_names.len(), expected_num_account_names_before);
        let new_account_names = account_names.remove(&get_sample_eos_erc20_account_name_2()).unwrap();
        assert_eq!(new_account_names.len(), expected_num_account_names_after);
    }

    #[test]
    fn should_save_account_names_in_db() {
        let db = get_test_database();
        let account_names = get_sample_eos_erc20_account_names();
        account_names.save_to_db(&db).unwrap();
        let result = db.get(EOS_ERC20_ACCOUNT_NAMES.to_vec(), MIN_DATA_SENSITIVITY_LEVEL).unwrap();
        assert_eq!(result, account_names.to_bytes().unwrap());
    }

    #[test]
    fn get_from_db_should_get_empty_account_names_if_non_extant() {
        let db = get_test_database();
        let expected_result = EosErc20AccountNames::new(vec![]);
        let result = EosErc20AccountNames::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_from_db_should_get_correct_account_names_if_extant() {
        let db = get_test_database();
        let account_names = get_sample_eos_erc20_account_names();
        account_names.save_to_db(&db).unwrap();
        let result = EosErc20AccountNames::get_from_db(&db).unwrap();
        assert_eq!(result, account_names);
    }

    #[test]
    fn should_add_and_update_in_db() {
        let db = get_test_database();
        let account_names = EosErc20AccountNames::new(vec![get_sample_eos_erc20_account_name_1()]);
        account_names.add_and_update_in_db(get_sample_eos_erc20_account_name_2(), &db).unwrap();
        let result = EosErc20AccountNames::get_from_db(&db).unwrap();
        assert_eq!(result, get_sample_eos_erc20_account_names());
    }

    #[test]
    fn should_remove_and_update_in_db() {
        let db = get_test_database();
        let account_names = get_sample_eos_erc20_account_names();
        account_names.save_to_db(&db).unwrap();
        account_names.remove_and_update_in_db(&get_sample_eos_erc20_account_name_1(), &db).unwrap();
        let result = EosErc20AccountNames::get_from_db(&db).unwrap();
        let expected_result = EosErc20AccountNames::new(vec![get_sample_eos_erc20_account_name_2()]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_account_name_from_token_address() {
        let token_address = EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let account_names = get_sample_eos_erc20_account_names();
        let expected_result = "SampleToken_1".to_string();
        let result = account_names.get_account_name_from_token_address(&token_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_err_when_get_account_name_from_token_address_if_not_extant() {
        let token_address = EthAddress::from_slice(&hex::decode("8f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap());
        let account_names = get_sample_eos_erc20_account_names();
        let result = account_names.get_account_name_from_token_address(&token_address);
        assert!(result.is_err());
    }

    #[test]
    fn should_return_true_if_token_supported() {
        let supported_token_address = EthAddress::from_slice(
            &hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()
        );
        let account_names = get_sample_eos_erc20_account_names();
        let result = account_names.is_token_supported(&supported_token_address);
        assert!(result);
    }

    #[test]
    fn should_return_false_if_token_supported() {
        let supported_token_address = EthAddress::from_slice(
            &hex::decode("8f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()
        );
        let account_names = get_sample_eos_erc20_account_names();
        let result = account_names.is_token_supported(&supported_token_address);
        assert!(!result);
    }
}
