use rust_algorand::{AlgorandAddress, AlgorandHash};

use crate::{
    chains::algo::algo_constants::{ALGO_GENESIS_HASH_KEY, ALGO_LATEST_BLOCK_NUMBER_KEY, ALGO_REDEEM_ADDRESS_KEY},
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    database_utils::{get_u64_from_db, put_u64_in_db},
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgoDbUtils<'a, D: DatabaseInterface> {
    db: &'a D,
    algo_redeem_address_key: Bytes,
    algo_genesis_hash_key: Bytes,
    algo_latest_block_number_key: Bytes,
}

impl<'a, D: DatabaseInterface> AlgoDbUtils<'a, D> {
    pub fn new(db: &'a D) -> Self {
        Self {
            db,
            algo_redeem_address_key: ALGO_REDEEM_ADDRESS_KEY.to_vec(),
            algo_genesis_hash_key: ALGO_GENESIS_HASH_KEY.to_vec(),
            algo_latest_block_number_key: ALGO_LATEST_BLOCK_NUMBER_KEY.to_vec(),
        }
    }

    fn get_no_put_again_error_msg(s: &str) -> String {
        format!("Cannot put ALGO {} in db - one is already there!", s)
    }

    fn get_db(&self) -> &D {
        self.db
    }

    pub fn get_latest_block_number(&self) -> Result<u64> {
        get_u64_from_db(self.get_db(), &self.algo_latest_block_number_key)
    }

    pub fn put_latest_block_number_in_db(&self, block_number: u64) -> Result<()> {
        put_u64_in_db(self.get_db(), &self.algo_latest_block_number_key, block_number)
    }

    pub fn put_genesis_hash_in_db(&self, genesis_hash: &AlgorandHash) -> Result<()> {
        if self.get_genesis_hash_from_db().is_ok() {
            Err(Self::get_no_put_again_error_msg("genesis hash").into())
        } else {
            self.get_db().put(
                self.algo_genesis_hash_key.clone(),
                genesis_hash.to_bytes(),
                MIN_DATA_SENSITIVITY_LEVEL,
            )
        }
    }

    pub fn get_genesis_hash_from_db(&self) -> Result<AlgorandHash> {
        self.get_db()
            .get(self.algo_genesis_hash_key.clone(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(AlgorandHash::from_bytes(&bytes)?))
    }

    pub fn put_redeem_address_in_db(&self, address: &AlgorandAddress) -> Result<()> {
        if self.get_redeem_address_from_db().is_ok() {
            Err(Self::get_no_put_again_error_msg("redeem address").into())
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
    fn should_put_and_get_algorand_genesis_hash_in_db() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let hash = AlgorandHash::from_bytes(&get_32_random_bytes_arr()).unwrap();
        db_utils.put_genesis_hash_in_db(&hash).unwrap();
        let result = db_utils.get_genesis_hash_from_db().unwrap();
        assert_eq!(result, hash);
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
}
