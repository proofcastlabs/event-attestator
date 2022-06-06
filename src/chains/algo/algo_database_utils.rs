#![allow(unused)] // NOTE: Not ALL fxns in here are required, but the macro is a better way to write them!

use std::{fmt, str::FromStr};

use paste::paste;
use rust_algorand::{AlgorandAddress, AlgorandAppId, AlgorandHash, AlgorandKeys, MicroAlgos};

use crate::{
    chains::algo::{
        algo_constants::{ALGO_PTOKEN_GENESIS_HASH, ALGO_TAIL_LENGTH},
        algo_submission_material::AlgoSubmissionMaterial,
    },
    constants::{MAX_DATA_SENSITIVITY_LEVEL, MIN_DATA_SENSITIVITY_LEVEL},
    database_utils::{get_u64_from_db, put_u64_in_db},
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::capitalize_first_letter,
};

create_db_utils_with_getters!(
    "Algo";
    "_fee_key" => "algo_fee_key",
    "_private_key_key" => "algo_private_key_key",
    "_account_nonce_key" => "algo_account_nonce_key",
    "_redeem_address_key" => "algo_redeem_address_key",
    "_tail_block_hash_key" => "algo_tail_block_hash_key",
    "_canon_block_hash_key" => "algo_canon_block_hash_key",
    "_linker_block_hash_key" => "algo_linker_block_hash_key",
    "_anchor_block_hash_key" => "algo_anchor_block_hash_key",
    "_latest_block_hash_key" => "algo_latest_block_hash_key",
    "_genesis_block_hash_key" => "algo_genesis_block_hash_key",
    "_canon_to_tip_length_key" => "algo_canon_to_tip_length_key",
    "_issuance_manager_app_id_key" => "algo_issuance_manager_app_id_key"
);

macro_rules! create_special_hash_setters_and_getters {
    ($($hash_type:expr),*) => {
        paste! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            enum SpecialHashTypes {
                $([< $hash_type:camel >],)*
            }

            impl SpecialHashTypes {
                fn get_key<D: DatabaseInterface>(&self, db_utils: &AlgoDbUtils<D>) -> Bytes {
                    match self {
                        $(Self::[< $hash_type:camel >] => db_utils.[<algo_ $hash_type _block_hash_key>].clone(),)*
                    }
                }
            }

            impl fmt::Display for SpecialHashTypes {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        $(Self::[< $hash_type:camel >]=> write!(f, $hash_type),)*
                    }
                }
            }

            impl FromStr for SpecialHashTypes {
                type Err = AppError;

                fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                    match s.to_lowercase().as_ref() {
                        $($hash_type => Ok(Self::[< $hash_type:camel >]),)*
                        _ => Err(format!("Unrecognized special hash type: '{}'", s).into()),
                    }
                }
            }

            $(
                impl<'a, D: DatabaseInterface> AlgoDbUtils<'a, D> {
                    pub fn [<get_ $hash_type _block_hash>](&self) -> Result<AlgorandHash> {
                        info!("✔ Getting {} block hash from db...", $hash_type);
                        self.get_special_hash_from_db(&SpecialHashTypes::from_str(&$hash_type)?)
                    }

                    pub fn [< put_ $hash_type _block_hash_in_db>](&self, hash: &AlgorandHash) -> Result<()> {
                        info!("✔ Putting {} block hash in db...", $hash_type);
                        self.put_special_hash_in_db(&SpecialHashTypes::from_str(&$hash_type)?, hash)
                    }

                    pub fn[<get_ $hash_type _submission_material>](&self) -> Result<AlgoSubmissionMaterial> {
                        info!("✔ Getting {} submission material from db...", $hash_type);
                        self.[< get_ $hash_type _block_hash>]()
                            .and_then(|hash| self.get_submission_material(&hash))
                    }

                    pub fn[<put_ $hash_type _submission_material_in_db>](
                        &self,
                        submission_material: &AlgoSubmissionMaterial
                    ) -> Result<()> {
                        info!("✔ Putting {} submission material in db!", $hash_type);
                        let block_hash = submission_material.block.hash()?;
                        self.put_algo_submission_material_in_db(submission_material)
                            .and_then(|_| self.[< put_ $hash_type _block_hash_in_db>](&block_hash))
                    }

                    pub fn [< get_ $hash_type _block_number >](&self) -> Result<u64> {
                        self
                            .[<get_ $hash_type _submission_material>]()
                            .map(|submission_material| submission_material.block.round())
                    }

                }
            )*

            #[cfg(test)]
            mod macro_tests {
                use super::*;
                use crate::{
                    test_utils::get_test_database,
                    chains::algo::test_utils::get_sample_submission_material_n,
                };

                $(
                    #[test]
                    fn [< should_put_and_get_ $hash_type _submission_material_in_db >]() {
                        let db = get_test_database();
                        let db_utils = AlgoDbUtils::new(&db);
                        let submission_material = get_sample_submission_material_n(0);
                        db_utils.[<put_ $hash_type _submission_material_in_db>](&submission_material).unwrap();
                        let result = db_utils.[<get_ $hash_type _submission_material>]().unwrap();
                        assert_eq!(result, submission_material);
                    }

                    #[test]
                    fn [<$hash_type _hash_should_be_set_correctly_when_adding_ $hash_type _block>]() {
                        let db = get_test_database();
                        let db_utils = AlgoDbUtils::new(&db);
                        let submission_material = get_sample_submission_material_n(0);
                        let hash = submission_material.block.hash().unwrap();
                        db_utils.[<put_ $hash_type _submission_material_in_db>](&submission_material).unwrap();
                        let result = db_utils.[<get_ $hash_type _block_hash>]().unwrap();
                        assert_eq!(result, hash);

                    }

                    #[test]
                    fn [<should_get_ $hash_type _block_number>]() {
                        let db = get_test_database();
                        let db_utils = AlgoDbUtils::new(&db);
                        let submission_material = get_sample_submission_material_n(0);
                        let expected_result = submission_material.block.round();
                        db_utils.[<put_ $hash_type _submission_material_in_db>](&submission_material).unwrap();
                        let result = db_utils.[<get_ $hash_type _block_number>]().unwrap();
                        assert_eq!(result, expected_result);
                    }
                )*
            }
        }
    }
}

create_special_hash_setters_and_getters!("tail", "canon", "anchor", "latest", "genesis", "linker");

impl<'a, D: DatabaseInterface> AlgoDbUtils<'a, D> {
    pub fn put_algo_app_id_in_db(&self, app_id: &AlgorandAppId) -> Result<()> {
        self.get_db().put(
            self.algo_issuance_manager_app_id_key.clone(),
            app_id.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_algo_app_id(&self) -> Result<AlgorandAppId> {
        self.get_db()
            .get(
                self.algo_issuance_manager_app_id_key.clone(),
                MIN_DATA_SENSITIVITY_LEVEL,
            )
            .and_then(|bytes| Ok(AlgorandAppId::from_bytes(&bytes)?))
    }

    pub fn get_genesis_hash(&self) -> Result<AlgorandHash> {
        info!("✔ Getting genesis hash from db...");
        self.get_db()
            .get(self.algo_genesis_block_hash_key.clone(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandHash::from_bytes(&bytes)?))
    }

    pub fn put_genesis_hash_in_db(&self, genesis_hash: &AlgorandHash) -> Result<()> {
        info!("✔ Putting genesis hash in db...");
        self.get_db().put(
            self.algo_genesis_block_hash_key.clone(),
            genesis_hash.to_bytes(),
            MAX_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_linker_hash_or_else_genesis_hash(&self) -> Result<AlgorandHash> {
        info!("✔ Getting linker hash or else genesis hash from db...");
        self.get_linker_block_hash().or(Ok(*ALGO_PTOKEN_GENESIS_HASH))
    }

    pub fn delete_submission_material_by_hash(&self, hash: &AlgorandHash) -> Result<()> {
        info!("Deleting block by blockhash: {}", hash);
        self.get_db().delete(hash.to_bytes())
    }

    fn maybe_get_algo_submission_material(&self, hash: &AlgorandHash) -> Option<AlgoSubmissionMaterial> {
        debug!("✔ Maybe getting ALGO submission material via hash: {}", hash);
        match self.get_submission_material(hash) {
            Ok(material) => Some(material),
            Err(_) => None,
        }
    }

    fn maybe_get_nth_ancestor_submission_material(
        &self,
        hash: &AlgorandHash,
        n: u64,
    ) -> Result<Option<AlgoSubmissionMaterial>> {
        debug!("✔ Getting ancestor #{} ALGO submission material from db...", n);
        match self.maybe_get_algo_submission_material(hash) {
            None => Ok(None),
            Some(material) => match n {
                0 => Ok(Some(material)),
                _ => self.maybe_get_nth_ancestor_submission_material(&material.block.get_previous_block_hash()?, n - 1),
            },
        }
    }

    fn maybe_get_candidate_submission_material(&self, ancestor_num: u64) -> Result<Option<AlgoSubmissionMaterial>> {
        self.maybe_get_nth_ancestor_submission_material(&self.get_latest_block_hash()?, ancestor_num)
    }

    pub fn maybe_get_new_canon_submission_material_candidate(&self) -> Result<Option<AlgoSubmissionMaterial>> {
        info!("✔ Maybe getting candidate canon block from db...");
        self.maybe_get_candidate_submission_material(self.get_canon_to_tip_length()?)
    }

    pub fn maybe_get_new_tail_block_candidate(&self) -> Result<Option<AlgoSubmissionMaterial>> {
        info!("✔ Maybe getting candidate tail block from db...");
        self.maybe_get_candidate_submission_material(self.get_canon_to_tip_length()? + ALGO_TAIL_LENGTH)
    }

    pub fn get_submission_material(&self, hash: &AlgorandHash) -> Result<AlgoSubmissionMaterial> {
        debug!("✔ Getting ALGO submission material via hash: {}", hash);
        self.get_db()
            .get(hash.to_bytes(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| AlgoSubmissionMaterial::from_bytes(&bytes))
    }

    pub fn get_algo_address(&self, key: &[Byte]) -> Result<AlgorandAddress> {
        info!("✔ Getting ALGO address from db...");
        self.get_db()
            .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandAddress::from_bytes(&bytes)?))
    }

    fn put_algo_address_in_db(&self, key: &[Byte], address: &AlgorandAddress) -> Result<()> {
        info!("✔ Putting ALGO address of {address} in db!");
        self.get_db()
            .put(key.to_vec(), address.to_bytes(), MIN_DATA_SENSITIVITY_LEVEL)
    }

    pub fn put_algo_submission_material_in_db(&self, submission_material: &AlgoSubmissionMaterial) -> Result<()> {
        info!("✔ Putting ALGO block in db...");
        self.get_db().put(
            submission_material.block.hash()?.to_bytes(),
            submission_material.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_submission_material_from_db(&self, hash: &AlgorandHash) -> Result<AlgoSubmissionMaterial> {
        debug!("✔ Getting ALGO submission material from db under hash {hash}");
        self.get_db()
            .get(hash.to_bytes(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| AlgoSubmissionMaterial::from_bytes(&bytes))
    }

    fn put_special_hash_in_db(&self, hash_type: &SpecialHashTypes, hash: &AlgorandHash) -> Result<()> {
        if hash_type == &SpecialHashTypes::Genesis && self.get_genesis_block_hash().is_ok() {
            return Err(Self::get_no_overwrite_error("genesis hash").into());
        };
        self.put_algorand_hash_in_db(&hash_type.get_key(self), hash)
    }

    pub fn put_algo_account_nonce_in_db(&self, nonce: u64) -> Result<()> {
        info!("✔ Putting ALGO account nonce of {nonce} in db!");
        put_u64_in_db(self.get_db(), &self.algo_account_nonce_key, nonce)
    }

    pub fn get_algo_account_nonce(&self) -> Result<u64> {
        info!("✔ Getting ALGO account nonce from db...");
        get_u64_from_db(self.get_db(), &self.algo_account_nonce_key)
    }

    pub fn get_algo_private_key(&self) -> Result<AlgorandKeys> {
        info!("✔ Getting ALGO private key from db...");
        self.get_db()
            .get(self.algo_private_key_key.clone(), MAX_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandKeys::from_bytes(&bytes)?))
    }

    pub fn put_algo_private_key_in_db(&self, key: &AlgorandKeys) -> Result<()> {
        if self.get_algo_private_key().is_ok() {
            Err(Self::get_no_overwrite_error("private key").into())
        } else {
            info!("✔ Putting ALGO private key in db...");
            self.get_db().put(
                self.algo_private_key_key.clone(),
                key.to_bytes(),
                MAX_DATA_SENSITIVITY_LEVEL,
            )
        }
    }

    pub fn get_algo_fee(&self) -> Result<MicroAlgos> {
        info!("✔ Getting ALGO fee from db...");
        Ok(MicroAlgos::from_algos(get_u64_from_db(
            self.get_db(),
            &self.algo_fee_key,
        )?)?)
    }

    pub fn put_algo_fee_in_db(&self, fee: MicroAlgos) -> Result<()> {
        info!("✔ Putting ALGO fee of {fee} in db!");
        put_u64_in_db(self.get_db(), &self.algo_fee_key, fee.to_algos())
    }

    pub fn put_canon_to_tip_length_in_db(&self, length: u64) -> Result<()> {
        info!("✔ Putting ALGO canon to tip length of {} in db...", length);
        put_u64_in_db(self.get_db(), &self.algo_canon_to_tip_length_key, length)
    }

    pub fn get_canon_to_tip_length(&self) -> Result<u64> {
        info!("✔ Getting ALGO canon to tip length from db...");
        get_u64_from_db(self.get_db(), &self.algo_canon_to_tip_length_key)
    }

    fn get_special_hash_from_db(&self, hash_type: &SpecialHashTypes) -> Result<AlgorandHash> {
        self.get_algorand_hash_from_db(&hash_type.get_key(self))
    }

    fn get_no_overwrite_error(s: &str) -> String {
        format!("Cannot overwrite ALGO {} in db - one already exists!", s)
    }

    fn put_algorand_hash_in_db(&self, key: &[Byte], hash: &AlgorandHash) -> Result<()> {
        self.get_db()
            .put(key.to_vec(), hash.to_bytes(), MIN_DATA_SENSITIVITY_LEVEL)
    }

    fn get_algorand_hash_from_db(&self, key: &[Byte]) -> Result<AlgorandHash> {
        self.get_db()
            .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandHash::from_bytes(&bytes)?))
    }

    pub fn put_redeem_address_in_db(&self, address: &AlgorandAddress) -> Result<()> {
        if self.get_redeem_address().is_ok() {
            Err(Self::get_no_overwrite_error("redeem address").into())
        } else {
            info!("✔ Putting ALGO redeem address in db...");
            self.put_algo_address_in_db(&self.algo_redeem_address_key, address)
        }
    }

    pub fn get_redeem_address(&self) -> Result<AlgorandAddress> {
        info!("✔ Getting ALGO redeem address from db...");
        self.get_algo_address(&self.algo_redeem_address_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::algo::test_utils::{
            get_all_sample_submission_material,
            get_sample_contiguous_submission_material,
            get_sample_submission_material_n,
        },
        crypto_utils::get_32_random_bytes_arr,
        test_utils::get_test_database,
    };

    fn get_random_algorand_hash() -> AlgorandHash {
        AlgorandHash::from_bytes(&get_32_random_bytes_arr()).unwrap()
    }

    #[test]
    fn should_put_and_get_algorand_redeem_address_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let address = AlgorandAddress::create_random().unwrap();
        db_utils.put_redeem_address_in_db(&address).unwrap();
        let result = db_utils.get_redeem_address().unwrap();
        assert_eq!(result, address);
    }

    #[test]
    fn should_put_and_get_special_hash_type_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let special_hash_type = SpecialHashTypes::Anchor;
        let hash = get_random_algorand_hash();
        db_utils.put_special_hash_in_db(&special_hash_type, &hash).unwrap();
        let result = db_utils.get_special_hash_from_db(&special_hash_type).unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_tail_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_tail_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_tail_block_hash().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_canon_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_canon_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_canon_block_hash().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_anchor_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_anchor_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_anchor_block_hash().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_latest_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_latest_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_latest_block_hash().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_genesis_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_genesis_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_genesis_block_hash().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_not_be_able_to_set_genesis_block_hash_if_already_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let genesis_hash = get_random_algorand_hash();
        db_utils.put_genesis_block_hash_in_db(&genesis_hash).unwrap();
        let hash_from_db = db_utils.get_genesis_block_hash().unwrap();
        assert_eq!(hash_from_db, genesis_hash);
        let new_hash = get_random_algorand_hash();
        let expected_error = "Cannot overwrite ALGO genesis hash in db - one already exists!";
        match db_utils.put_genesis_block_hash_in_db(&new_hash) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        };
        let result = db_utils.get_genesis_block_hash().unwrap();
        assert_eq!(result, genesis_hash);
    }

    #[test]
    fn should_put_and_get_algo_canon_to_tip_length_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let length = 42;
        db_utils.put_canon_to_tip_length_in_db(42).unwrap();
        let result = db_utils.get_canon_to_tip_length().unwrap();
        assert_eq!(result, length);
    }

    #[test]
    fn should_put_and_get_algo_fee_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let fee = MicroAlgos::new(1000);
        db_utils.put_algo_fee_in_db(fee).unwrap();
        let result = db_utils.get_algo_fee().unwrap();
        assert_eq!(result, fee);
    }

    #[test]
    fn should_put_and_get_algorand_private_key_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let keys = AlgorandKeys::create_random();
        db_utils.put_algo_private_key_in_db(&keys).unwrap();
        let result = db_utils.get_algo_private_key().unwrap();
        assert_eq!(result, keys);
    }

    #[test]
    fn should_not_allow_overwrite_of_private_key() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let keys = AlgorandKeys::create_random();
        db_utils.put_algo_private_key_in_db(&keys).unwrap();
        let keys_from_db = db_utils.get_algo_private_key().unwrap();
        assert_eq!(keys_from_db, keys);
        let new_keys = AlgorandKeys::create_random();
        assert_ne!(keys, new_keys);
        let expected_error = "Cannot overwrite ALGO private key in db - one already exists!";
        match db_utils.put_algo_private_key_in_db(&new_keys) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
        let result = db_utils.get_algo_private_key().unwrap();
        assert_eq!(result, keys);
    }

    #[test]
    fn should_put_and_get_algo_account_nonce_from_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let nonce = 666;
        db_utils.put_algo_account_nonce_in_db(nonce).unwrap();
        let result = db_utils.get_algo_account_nonce().unwrap();
        assert_eq!(result, nonce);
    }

    #[test]
    fn algo_db_keys_should_remain_consistent() {
        #[rustfmt::skip]
        let expected_result = AlgoDatabaseKeysJson {
            ALGO_FEE_KEY:
                "d284e359e0a2076c909ee55d8deaf1e05b5488a997f18bf86e0928c4fbc5c638".to_string(),
            ALGO_REDEEM_ADDRESS_KEY:
                "6e4a528af852818a2f5c1660679873fbe3a49ab57ecf14bf0f542220e95cc6d4".to_string(),
            ALGO_TAIL_BLOCK_HASH_KEY:
                "2a307fe54ac8b580e12772152a6be38285afb11a932ab817c423a580c474fb3f".to_string(),
            ALGO_CANON_BLOCK_HASH_KEY:
                "1a4b2db39e866baa1e76f114c6620a94e7cd078bf1c81f5cd286e4213ea60892".to_string(),
            ALGO_ANCHOR_BLOCK_HASH_KEY:
                "0708c1e329a262c9ce0e39d91a05be6dbb270861869b2c48d8aa4d8e7aa58c75".to_string(),
            ALGO_LATEST_BLOCK_HASH_KEY:
                "d5743e9bee45679ce65bf04dc3fbce27ef1f148a13a37e4234288f92d3e2e124".to_string(),
            ALGO_GENESIS_BLOCK_HASH_KEY:
                "e10b845e685c345196e1b4f41a91fa74fc8ae7f000184f222f4b5df649b50585".to_string(),
            ALGO_CANON_TO_TIP_LENGTH_KEY:
                "295dafb37cf7d99e712b44c066951b962bef0243abb56b5aba1172ea70bfb5f5".to_string(),
            ALGO_PRIVATE_KEY_KEY:
                "90c457a020ebe52f3de54b258d3494466d30ee5b95bb1245da06546738ef80ff".to_string(),
            ALGO_ACCOUNT_NONCE_KEY:
                "805e14a1f236eac2b388f2cb625af8bacd8633cb489e84df62b99fbc80b28a0d".to_string(),
            ALGO_LINKER_BLOCK_HASH_KEY:
                "6a5d622179feb8e0b51f30517735aeb6cb1ded767e1868b527475bc7649a2d02".to_string(),
            ALGO_ISSUANCE_MANAGER_APP_ID_KEY:
                "a921bc2a9ca1fed67d74b19b97bce679457191f5f6684facc422d838dc3d275b".to_string(),
        };
        let result = AlgoDatabaseKeysJson::new();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_put_and_get_submission_material_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_material = AlgoSubmissionMaterial::default();
        db_utils
            .put_algo_submission_material_in_db(&submission_material)
            .unwrap();
        let result = db_utils
            .get_submission_material_from_db(&submission_material.block.hash().unwrap())
            .unwrap();
        assert_eq!(result, submission_material);
    }

    #[test]
    fn maybe_get_algo_submission_material_should_get_extant_submission_material() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_material = get_sample_submission_material_n(0);
        let hash = submission_material.block.hash().unwrap();
        db_utils
            .put_algo_submission_material_in_db(&submission_material)
            .unwrap();
        let result = db_utils.maybe_get_algo_submission_material(&hash);
        let expected_result = Some(submission_material);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn maybe_get_algo_submission_material_should_return_none_if_no_submission_material_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_material = get_sample_submission_material_n(0);
        let hash = submission_material.block.hash().unwrap();
        let result = db_utils.maybe_get_algo_submission_material(&hash);
        let expected_result = None;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_nth_ancestor_submission_material() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_all_sample_submission_material();
        submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(&material).unwrap());
        let sample_number = 4;
        let hash = get_sample_submission_material_n(sample_number).block.hash().unwrap();
        let n: u64 = 3;
        let expected_result = Some(get_sample_submission_material_n(sample_number - n as usize));
        let result = db_utils.maybe_get_nth_ancestor_submission_material(&hash, n).unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_return_none_if_no_nth_ancestor() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_all_sample_submission_material();
        submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(&material).unwrap());
        let hash = get_sample_submission_material_n(4).block.hash().unwrap();
        let expected_result = None;
        let result = db_utils
            .maybe_get_nth_ancestor_submission_material(&hash, (submission_materials.len() + 1) as u64)
            .unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_get_new_candidate_submission_material_if_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_sample_contiguous_submission_material();
        let canon_to_tip_length = (submission_materials.len() - 1) as u64;
        let latest_submission_material = submission_materials[submission_materials.len() - 1].clone();
        db_utils
            .put_latest_submission_material_in_db(&latest_submission_material)
            .unwrap();
        db_utils.put_canon_to_tip_length_in_db(canon_to_tip_length).unwrap();
        submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(&material).unwrap());
        let result = db_utils
            .maybe_get_candidate_submission_material(canon_to_tip_length)
            .unwrap();
        let expected_result =
            Some(submission_materials[submission_materials.len() - 1 - canon_to_tip_length as usize].clone());
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_get_new_candidate_block_if_not_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_sample_contiguous_submission_material();
        let canon_to_tip_length = (submission_materials.len() + 1) as u64;
        let submission_material = submission_materials[submission_materials.len() - 1].clone();
        db_utils
            .put_latest_submission_material_in_db(&submission_material)
            .unwrap();
        db_utils.put_canon_to_tip_length_in_db(canon_to_tip_length).unwrap();
        submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(&material).unwrap());
        let result = db_utils
            .maybe_get_candidate_submission_material(canon_to_tip_length)
            .unwrap();
        let expected_result = None;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_delete_submission_material_by_hash() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_material = get_sample_submission_material_n(0);
        let hash = submission_material.block.hash().unwrap();
        db_utils
            .put_algo_submission_material_in_db(&submission_material)
            .unwrap();
        assert!(db_utils.get_submission_material(&hash).is_ok());
        db_utils.delete_submission_material_by_hash(&hash);
        let result = db_utils.get_submission_material(&hash);
        assert!(result.is_err());
    }

    #[test]
    fn should_get_linker_hash_if_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        assert_ne!(hash, *ALGO_PTOKEN_GENESIS_HASH);
        db_utils.put_linker_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_linker_hash_or_else_genesis_hash().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn get_linker_hash_should_fall_back_to_genesis_hash_if_not_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        assert!(db_utils.get_linker_block_hash().is_err());
        let result = db_utils.get_linker_hash_or_else_genesis_hash().unwrap();
        assert_eq!(result, *ALGO_PTOKEN_GENESIS_HASH);
    }

    #[test]
    fn should_put_and_get_genesis_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = AlgorandHash::default();
        db_utils.put_genesis_hash_in_db(&hash).unwrap();
        let result = db_utils.get_genesis_hash().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_get_and_put_algo_app_id_in_db() {
        let app_id = AlgorandAppId::new(1337);
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        db_utils.put_algo_app_id_in_db(&app_id).unwrap();
        let result = db_utils.get_algo_app_id().unwrap();
        assert_eq!(result, app_id);
    }
}
