#![allow(unused)] // FIXME Rm!

use std::{fmt, str::FromStr};

use paste::paste;
use rust_algorand::{AlgorandAddress, AlgorandBlock, AlgorandHash, AlgorandKeys};

use crate::{
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
    "_anchor_block_hash_key" => "algo_anchor_block_hash_key",
    "_latest_block_hash_key" => "algo_latest_block_hash_key",
    "_genesis_block_hash_key" => "algo_genesis_block_hash_key",
    "_canon_to_tip_length_key" => "algo_canon_to_tip_length_key"
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

                    pub fn[<get_ $hash_type _block>](&self) -> Result<AlgorandBlock> {
                        info!("✔ Getting {} block from db...", $hash_type);
                        self.[< get_ $hash_type _block_hash>]()
                            .and_then(|hash| self.get_block(&hash))
                    }

                    pub fn[<put_ $hash_type _block_in_db>](&self, block: &AlgorandBlock) -> Result<()> {
                        info!("✔ Putting {} block in db!", $hash_type);
                        let block_hash = block.hash()?;
                        self.put_block_in_db(block)
                            .and_then(|_| self.[< put_ $hash_type _block_hash_in_db>](&block_hash))
                    }

                    pub fn [< get_ $hash_type _block_number >](&self) -> Result<u64> {
                        self.[<get_ $hash_type _block>]().map(|block| block.round())
                    }

                }
            )*

            #[cfg(test)]
            mod macro_tests {
                use super::*;
                use crate::{
                    test_utils::get_test_database,
                    chains::algo::test_utils::get_sample_block_n,
                };

                $(
                    #[test]
                    fn [< should_put_and_get_ $hash_type _block_in_db >]() {
                        let db = get_test_database();
                        let db_utils = AlgoDbUtils::new(&db);
                        let block = get_sample_block_n(0);
                        db_utils.[<put_ $hash_type _block_in_db>](&block).unwrap();
                        let result = db_utils.[<get_ $hash_type _block>]().unwrap();
                        assert_eq!(result, block);
                    }

                    #[test]
                    fn [<$hash_type _hash_should_be_set_correctly_when_adding_ $hash_type _block>]() {
                        let db = get_test_database();
                        let db_utils = AlgoDbUtils::new(&db);
                        let block = get_sample_block_n(0);
                        let hash = block.hash().unwrap();
                        db_utils.[<put_ $hash_type _block_in_db>](&block).unwrap();
                        let result = db_utils.[<get_ $hash_type _block_hash>]().unwrap();
                        assert_eq!(result, hash);

                    }

                    #[test]
                    fn [<should_get_ $hash_type _block_number>]() {
                        let db = get_test_database();
                        let db_utils = AlgoDbUtils::new(&db);
                        let block = get_sample_block_n(0);
                        let expected_result = block.round();
                        db_utils.[<put_ $hash_type _block_in_db>](&block).unwrap();
                        let result = db_utils.[<get_ $hash_type _block_number>]().unwrap();
                        assert_eq!(result, expected_result);
                    }
                )*
            }
        }
    }
}

create_special_hash_setters_and_getters!("tail", "canon", "anchor", "latest", "genesis");

impl<'a, D: DatabaseInterface> AlgoDbUtils<'a, D> {
    fn maybe_get_block(&self, hash: &AlgorandHash) -> Option<AlgorandBlock> {
        debug!("✔ Maybe getting ALGO block via hash: {}", hash);
        match self.get_block(hash) {
            Ok(block) => Some(block),
            Err(_) => None,
        }
    }

    fn maybe_get_nth_ancestor_block(&self, hash: &AlgorandHash, n: u64) -> Result<Option<AlgorandBlock>> {
        info!("✔ Getting {}th ancestor ALGO block from db...", n);
        match self.maybe_get_block(hash) {
            None => Ok(None),
            Some(block) => match n {
                0 => Ok(Some(block)),
                _ => self.maybe_get_nth_ancestor_block(&block.get_previous_block_hash()?, n - 1),
            },
        }
    }

    fn get_block(&self, hash: &AlgorandHash) -> Result<AlgorandBlock> {
        debug!("✔ Getting ALGO block via hash: {}", hash);
        self.get_db()
            .get(hash.to_bytes(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandBlock::from_bytes(&bytes)?))
    }

    fn get_algo_address_from_db(&self, key: &[Byte]) -> Result<AlgorandAddress> {
        self.get_db()
            .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandAddress::from_bytes(&bytes)?))
    }

    fn put_algo_address_in_db(&self, key: &[Byte], address: &AlgorandAddress) -> Result<()> {
        self.get_db()
            .put(key.to_vec(), address.to_bytes()?, MIN_DATA_SENSITIVITY_LEVEL)
    }

    fn put_block_in_db(&self, block: &AlgorandBlock) -> Result<()> {
        self.get_db()
            .put(block.hash()?.to_bytes(), block.to_bytes()?, MIN_DATA_SENSITIVITY_LEVEL)
    }

    fn get_block_from_db(&self, hash: &AlgorandHash) -> Result<AlgorandBlock> {
        self.get_db()
            .get(hash.to_bytes(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandBlock::from_bytes(&bytes)?))
    }

    fn put_special_hash_in_db(&self, hash_type: &SpecialHashTypes, hash: &AlgorandHash) -> Result<()> {
        if hash_type == &SpecialHashTypes::Genesis {
            if self.get_genesis_block_hash().is_ok() {
                return Err(Self::get_no_overwrite_error("genesis hash").into());
            }
        };
        self.put_algorand_hash_in_db(&hash_type.get_key(self), hash)
    }

    fn put_algo_account_nonce_in_db(&self, nonce: u64) -> Result<()> {
        put_u64_in_db(self.get_db(), &self.algo_account_nonce_key, nonce)
    }

    fn get_algo_account_nonce_from_db(&self) -> Result<u64> {
        get_u64_from_db(self.get_db(), &self.algo_account_nonce_key)
    }

    fn get_algo_private_key_from_db(&self) -> Result<AlgorandKeys> {
        self.get_db()
            .get(self.algo_private_key_key.clone(), MAX_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandKeys::from_bytes(&bytes)?))
    }

    fn put_algo_private_key_in_db(&self, key: &AlgorandKeys) -> Result<()> {
        if self.get_algo_private_key_from_db().is_ok() {
            Err(Self::get_no_overwrite_error("private key").into())
        } else {
            self.get_db().put(
                self.algo_private_key_key.clone(),
                key.to_bytes(),
                MAX_DATA_SENSITIVITY_LEVEL,
            )
        }
    }

    fn get_algo_fee_from_db(&self) -> Result<u64> {
        get_u64_from_db(self.get_db(), &self.algo_fee_key)
    }

    fn put_algo_fee_in_db(&self, fee: u64) -> Result<()> {
        put_u64_in_db(self.get_db(), &self.algo_fee_key, fee)
    }

    fn put_canon_to_tip_length_in_db(&self, length: u64) -> Result<()> {
        put_u64_in_db(self.get_db(), &self.algo_canon_to_tip_length_key, length)
    }

    fn get_canon_to_tip_length_from_db(&self) -> Result<u64> {
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
        if self.get_redeem_address_from_db().is_ok() {
            Err(Self::get_no_overwrite_error("redeem address").into())
        } else {
            self.put_algo_address_in_db(&self.algo_redeem_address_key, address)
        }
    }

    pub fn get_redeem_address_from_db(&self) -> Result<AlgorandAddress> {
        self.get_algo_address_from_db(&self.algo_redeem_address_key)
    }

    pub fn get_latest_block_number(&self) -> Result<u64> {
        self.get_latest_block().map(|block| block.round())
    }

    pub fn get_public_algo_address_from_db(&self) -> Result<AlgorandAddress> {
        // TODO
        unimplemented!()
        //Ok(AlgorandAddress::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::algo::test_utils::{get_all_sample_blocks, get_sample_block_n},
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
        let result = db_utils.get_redeem_address_from_db().unwrap();
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
    fn should_not_be_able_to_set_genesis_block_hash_if_alreadyt_extant() {
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
        let result = db_utils.get_canon_to_tip_length_from_db().unwrap();
        assert_eq!(result, length);
    }

    #[test]
    fn should_put_and_get_algo_fee_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let fee = 1000;
        db_utils.put_algo_fee_in_db(fee).unwrap();
        let result = db_utils.get_algo_fee_from_db().unwrap();
        assert_eq!(result, fee);
    }

    #[test]
    fn should_put_and_get_algorand_private_key_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let keys = AlgorandKeys::create_random();
        db_utils.put_algo_private_key_in_db(&keys).unwrap();
        let result = db_utils.get_algo_private_key_from_db().unwrap();
        assert_eq!(result, keys);
    }

    #[test]
    fn should_not_allow_overwrite_of_private_key() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let keys = AlgorandKeys::create_random();
        db_utils.put_algo_private_key_in_db(&keys).unwrap();
        let keys_from_db = db_utils.get_algo_private_key_from_db().unwrap();
        assert_eq!(keys_from_db, keys);
        let new_keys = AlgorandKeys::create_random();
        assert_ne!(keys, new_keys);
        let expected_error = "Cannot overwrite ALGO private key in db - one already exists!";
        match db_utils.put_algo_private_key_in_db(&new_keys) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
        let result = db_utils.get_algo_private_key_from_db().unwrap();
        assert_eq!(result, keys);
    }

    #[test]
    fn should_put_and_get_algo_account_nonce_from_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let nonce = 666;
        db_utils.put_algo_account_nonce_in_db(nonce).unwrap();
        let result = db_utils.get_algo_account_nonce_from_db().unwrap();
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
        };
        let result = AlgoDatabaseKeysJson::new();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_put_and_get_block_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let block = AlgorandBlock::default();
        db_utils.put_block_in_db(&block).unwrap();
        let result = db_utils.get_block_from_db(&block.hash().unwrap()).unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn maybe_get_block_should_get_extant_block() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let block = get_sample_block_n(0);
        let hash = block.hash().unwrap();
        db_utils.put_block_in_db(&block).unwrap();
        let result = db_utils.maybe_get_block(&hash);
        let expected_result = Some(block);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn maybe_get_block_should_return_none_if_no_block_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let block = get_sample_block_n(0);
        let hash = block.hash().unwrap();
        let result = db_utils.maybe_get_block(&hash);
        let expected_result = None;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_nth_ancestor_block() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let blocks = get_all_sample_blocks();
        blocks
            .iter()
            .for_each(|block| db_utils.put_block_in_db(&block).unwrap());
        let block_sample_number = 4;
        let hash = get_sample_block_n(block_sample_number).hash().unwrap();
        let n: u64 = 3;
        let expected_result = Some(get_sample_block_n(block_sample_number - n as usize));
        let result = db_utils.maybe_get_nth_ancestor_block(&hash, n).unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_return_none_if_no_nth_ancestor() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let blocks = get_all_sample_blocks();
        blocks
            .iter()
            .for_each(|block| db_utils.put_block_in_db(&block).unwrap());
        let hash = get_sample_block_n(4).hash().unwrap();
        let expected_result = None;
        let result = db_utils
            .maybe_get_nth_ancestor_block(&hash, (blocks.len() + 1) as u64)
            .unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_get_latest_block_number() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let block = get_sample_block_n(0);
        let expected_result = block.round();
        db_utils.put_latest_block_in_db(&block).unwrap();
        let result = db_utils.get_latest_block_number().unwrap();
        assert_eq!(result, expected_result);
    }
}
