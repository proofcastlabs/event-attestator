/* Setters and getters TODO
   [x] chain id
   [x] save anchor block hash
   [x] save latest block hash
   [x] save canon block hash
   [x] address
   [x] tail block hash
   [x] save canon to tip length
   [x] fee
   [ ] private key
   [ ] account nonce
   [ ] block
*/
use std::{fmt, str::FromStr};

use paste::paste;
use rust_algorand::{AlgorandAddress, AlgorandHash, AlgorandKeys};

use crate::{
    constants::{MAX_DATA_SENSITIVITY_LEVEL, MIN_DATA_SENSITIVITY_LEVEL},
    database_utils::{get_u64_from_db, put_u64_in_db},
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::capitalize_first_letter,
};

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
                    pub fn [<get_ $hash_type _block_hash_from_db>](&self) -> Result<AlgorandHash> {
                        self.get_special_hash_from_db(&SpecialHashTypes::from_str(&$hash_type)?)
                    }

                    pub fn [< put_ $hash_type _block_hash_in_db>](&self, hash: &AlgorandHash) -> Result<()> {
                        self.put_special_hash_in_db(&SpecialHashTypes::from_str(&$hash_type)?, hash)
                    }
                }
            )*
        }
    }
}

macro_rules! create_algo_db_utils {
    ($($name:expr),*) => {
        paste! {
            use crate::chains::algo::algo_constants::{
                $([< $name:upper >],)*
            };

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct AlgoDbUtils<'a, D: DatabaseInterface> {
                db: &'a D,
                $([< $name >]: Bytes,)*
            }

            impl<'a, D: DatabaseInterface> AlgoDbUtils<'a, D> {
                pub fn new(db: &'a D) -> Self {
                    Self {
                        db,
                        $([< $name >]: [< $name:upper >].to_vec(),)*
                    }
                }
            }
        }
    }
}

create_algo_db_utils!(
    "algo_fee_key",
    "algo_private_key_key",
    "algo_redeem_address_key",
    "algo_tail_block_hash_key",
    "algo_canon_block_hash_key",
    "algo_latest_block_hash_key",
    "algo_anchor_block_hash_key",
    "algo_genesis_block_hash_key",
    "algo_latest_block_number_key",
    "algo_canon_to_tip_length_key"
);

create_special_hash_setters_and_getters!("tail", "canon", "anchor", "latest", "genesis");

impl<'a, D: DatabaseInterface> AlgoDbUtils<'a, D> {
    fn put_special_hash_in_db(&self, hash_type: &SpecialHashTypes, hash: &AlgorandHash) -> Result<()> {
        if hash_type == &SpecialHashTypes::Genesis {
            if self.get_genesis_block_hash_from_db().is_ok() {
                return Err(Self::get_no_overwrite_error("genesis hash").into());
            }
        };
        self.put_algorand_hash_in_db(&hash_type.get_key(&self), hash)
    }

    fn get_algo_private_key_from_db(&self) -> Result<AlgorandKeys> {
        self
            .get_db()
            .get(self.algo_private_key_key.clone(), MAX_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandKeys::from_bytes(&bytes)?))

    }

    fn put_algo_private_key_in_db(&self, key: &AlgorandKeys) -> Result<()> {
        if self.get_algo_private_key_from_db().is_ok() {
            Err(Self::get_no_overwrite_error("private key").into())
        } else {
            self.get_db().put(self.algo_private_key_key.clone(), key.to_bytes(), MAX_DATA_SENSITIVITY_LEVEL)
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
        self.get_algorand_hash_from_db(&hash_type.get_key(&self))
    }

    fn get_no_overwrite_error(s: &str) -> String {
        format!("Cannot overwrite ALGO {} in db - one already exists!", s)
    }

    fn get_db(&self) -> &D {
        self.db
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

    pub fn get_latest_block_number(&self) -> Result<u64> {
        get_u64_from_db(self.get_db(), &self.algo_latest_block_number_key)
    }

    pub fn put_latest_block_number_in_db(&self, block_number: u64) -> Result<()> {
        put_u64_in_db(self.get_db(), &self.algo_latest_block_number_key, block_number)
    }

    pub fn put_redeem_address_in_db(&self, address: &AlgorandAddress) -> Result<()> {
        if self.get_redeem_address_from_db().is_ok() {
            Err(Self::get_no_overwrite_error("redeem address").into())
        } else {
            self.get_db().put(
                self.algo_redeem_address_key.clone(),
                address.to_bytes()?,
                MIN_DATA_SENSITIVITY_LEVEL,
            )
        }
    }

    pub fn get_redeem_address_from_db(&self) -> Result<AlgorandAddress> {
        self.get_db()
            .get(self.algo_redeem_address_key.clone(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandAddress::from_bytes(&bytes)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{crypto_utils::get_32_random_bytes_arr, test_utils::get_test_database};

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
    fn should_put_and_get_algorand_latet_block_number() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let number = 1337;
        db_utils.put_latest_block_number_in_db(number).unwrap();
        let result = db_utils.get_latest_block_number().unwrap();
        assert_eq!(result, number);
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
        let result = db_utils.get_tail_block_hash_from_db().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_canon_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_canon_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_canon_block_hash_from_db().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_anchor_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_anchor_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_anchor_block_hash_from_db().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_latest_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_latest_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_latest_block_hash_from_db().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_put_and_get_algorand_genesis_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = get_random_algorand_hash();
        db_utils.put_genesis_block_hash_in_db(&hash).unwrap();
        let result = db_utils.get_genesis_block_hash_from_db().unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_not_be_able_to_set_genesis_block_hash_if_alreadyt_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let genesis_hash = get_random_algorand_hash();
        db_utils.put_genesis_block_hash_in_db(&genesis_hash).unwrap();
        let hash_from_db = db_utils.get_genesis_block_hash_from_db().unwrap();
        assert_eq!(hash_from_db, genesis_hash);
        let new_hash = get_random_algorand_hash();
        let expected_error = "Cannot overwrite ALGO genesis hash in db - one already exists!";
        match db_utils.put_genesis_block_hash_in_db(&new_hash) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        };
        let result = db_utils.get_genesis_block_hash_from_db().unwrap();
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
}
