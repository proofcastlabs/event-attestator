#![allow(dead_code)] // FIXME rm!

use bitcoin::{hashes::Hash, network::constants::Network as BtcNetwork, BlockHash};

use crate::{
    chains::btc::{
        btc_block::BtcBlockInDbFormat,
        btc_chain_id::BtcChainId,
        btc_crypto::btc_private_key::BtcPrivateKey,
        btc_state::BtcState,
        btc_types::BtcPubKeySlice,
        btc_utils::{convert_btc_address_to_bytes, convert_bytes_to_btc_address, convert_bytes_to_btc_pub_key_slice},
    },
    constants::{MAX_DATA_SENSITIVITY_LEVEL, MIN_DATA_SENSITIVITY_LEVEL},
    database_utils::{get_u64_from_db, put_u64_in_db},
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, DataSensitivity, Result},
    utils::{convert_bytes_to_u64, convert_u64_to_bytes},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtcDatabaseUtils<'a, D: DatabaseInterface> {
    db: &'a D,
    btc_fee_key: Bytes,
    btc_network_key: Bytes,
    btc_address_key: Bytes,
    btc_linker_hash_key: Bytes,
    btc_public_key_db_key: Bytes,
    btc_account_nonce_key: Bytes,
    btc_private_key_db_key: Bytes,
    btc_tail_block_hash_key: Bytes,
    btc_difficulty_threshold: Bytes,
    btc_canon_block_hash_key: Bytes,
    btc_anchor_block_hash_key: Bytes,
    btc_latest_block_hash_key: Bytes,
    btc_canon_to_tip_length_key: Bytes,
}

impl<'a, D: DatabaseInterface> BtcDatabaseUtils<'a, D> {
    pub fn new(db: &'a D) -> Self {
        use crate::chains::btc::btc_constants::{
            BTC_ACCOUNT_NONCE_KEY,
            BTC_ADDRESS_KEY,
            BTC_ANCHOR_BLOCK_HASH_KEY,
            BTC_CANON_BLOCK_HASH_KEY,
            BTC_CANON_TO_TIP_LENGTH_KEY,
            BTC_DIFFICULTY_THRESHOLD,
            BTC_FEE_KEY,
            BTC_LATEST_BLOCK_HASH_KEY,
            BTC_LINKER_HASH_KEY,
            BTC_NETWORK_KEY,
            BTC_PRIVATE_KEY_DB_KEY,
            BTC_PUBLIC_KEY_DB_KEY,
            BTC_TAIL_BLOCK_HASH_KEY,
        };
        Self {
            db,
            btc_fee_key: BTC_FEE_KEY.to_vec(),
            btc_network_key: BTC_NETWORK_KEY.to_vec(),
            btc_address_key: BTC_ADDRESS_KEY.to_vec(),
            btc_linker_hash_key: BTC_LINKER_HASH_KEY.to_vec(),
            btc_public_key_db_key: BTC_PUBLIC_KEY_DB_KEY.to_vec(),
            btc_account_nonce_key: BTC_ACCOUNT_NONCE_KEY.to_vec(),
            btc_private_key_db_key: BTC_PRIVATE_KEY_DB_KEY.to_vec(),
            btc_tail_block_hash_key: BTC_TAIL_BLOCK_HASH_KEY.to_vec(),
            btc_difficulty_threshold: BTC_DIFFICULTY_THRESHOLD.to_vec(),
            btc_canon_block_hash_key: BTC_CANON_BLOCK_HASH_KEY.to_vec(),
            btc_anchor_block_hash_key: BTC_ANCHOR_BLOCK_HASH_KEY.to_vec(),
            btc_latest_block_hash_key: BTC_LATEST_BLOCK_HASH_KEY.to_vec(),
            btc_canon_to_tip_length_key: BTC_CANON_TO_TIP_LENGTH_KEY.to_vec(),
        }
    }

    fn put_btc_pub_key_slice_in_db(&self, pub_key_slice: &BtcPubKeySlice) -> Result<()> {
        self.db.put(
            self.btc_public_key_db_key.clone(),
            pub_key_slice.to_vec(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_btc_public_key_slice_from_db(&self) -> Result<BtcPubKeySlice> {
        self.db
            .get(self.btc_public_key_db_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_btc_pub_key_slice(&bytes))
    }

    fn increment_btc_account_nonce_in_db(&self, amount_to_increment_by: u64) -> Result<()> {
        info!("✔ Incrementing BTC account nonce in db...");
        self.get_btc_account_nonce_from_db()
            .and_then(|nonce| self.put_btc_account_nonce_in_db(nonce + amount_to_increment_by))
    }

    fn put_btc_account_nonce_in_db(&self, nonce: u64) -> Result<()> {
        debug!("✔ Putting BTC account nonce of {} in db...", nonce);
        put_u64_in_db(self.db, &self.btc_account_nonce_key.to_vec(), nonce)
    }

    fn get_btc_account_nonce_from_db(&self) -> Result<u64> {
        debug!("✔ Getting BTC account nonce from db...");
        get_u64_from_db(self.db, &self.btc_account_nonce_key.to_vec())
    }

    fn get_btc_fee_from_db(&self) -> Result<u64> {
        debug!("✔ Getting BTC fee from db...");
        self.db
            .get(self.btc_fee_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_u64(&bytes))
    }

    fn put_btc_fee_in_db(&self, fee: u64) -> Result<()> {
        debug!("✔ Adding BTC fee of '{}' satoshis-per-byte to db...", fee);
        self.db.put(
            self.btc_fee_key.to_vec(),
            convert_u64_to_bytes(fee),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_btc_network_from_db(&self) -> Result<BtcNetwork> {
        self.db
            .get(self.btc_network_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|ref bytes| BtcChainId::from_bytes(bytes))
            .map(|chain_id| chain_id.to_btc_network())
    }

    fn put_btc_network_in_db(&self, network: BtcNetwork) -> Result<()> {
        info!("✔ Adding BTC '{}' network to database...", network);
        self.db.put(
            self.btc_network_key.to_vec(),
            BtcChainId::from_btc_network(&network)?.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn put_btc_difficulty_in_db(&self, difficulty: u64) -> Result<()> {
        debug!("✔ Putting BTC difficulty threshold of {} in db...", difficulty);
        self.db.put(
            self.btc_difficulty_threshold.to_vec(),
            convert_u64_to_bytes(difficulty),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_btc_difficulty_from_db(&self) -> Result<u64> {
        debug!("✔ Getting BTC difficulty threshold from db...");
        self.db
            .get(self.btc_difficulty_threshold.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_u64(&bytes))
    }

    fn get_btc_block_from_db(&self, id: &BlockHash) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC block from db via id: {}", hex::encode(id.to_vec()));
        self.db
            .get(id.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| BtcBlockInDbFormat::from_bytes(&bytes))
    }

    fn get_special_btc_block_from_db(&self, block_type: &str) -> Result<BtcBlockInDbFormat> {
        self.get_special_hash_from_db(block_type)
            .and_then(|block_hash| self.get_btc_block_from_db(&block_hash))
    }

    fn get_btc_hash_from_db(&self, key: &[Byte]) -> Result<BlockHash> {
        self.db
            .get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(BlockHash::from_slice(&bytes)?))
    }

    fn get_special_hash_from_db(&self, hash_type: &str) -> Result<BlockHash> {
        // FIXME/TODO make the block type an enum!
        let key = match hash_type {
            "tail" => Ok(self.btc_tail_block_hash_key.to_vec()),
            "canon" => Ok(self.btc_canon_block_hash_key.to_vec()),
            "anchor" => Ok(self.btc_anchor_block_hash_key.to_vec()),
            "latest" => Ok(self.btc_latest_block_hash_key.to_vec()),
            _ => Err(AppError::Custom(format!(
                "✘ Cannot get special BTC hash of type: {}!",
                hash_type
            ))),
        }?;
        debug!("✔ Getting special BTC hash from db of type: {}", hash_type);
        self.get_btc_hash_from_db(&key.to_vec())
    }

    fn get_btc_latest_block_from_db(&self) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC latest block from db...");
        self.get_special_btc_block_from_db("latest")
    }

    fn get_latest_btc_block_number(&self) -> Result<u64> {
        debug!("✔ Getting BTC latest block number from db...");
        self.get_btc_latest_block_from_db()
            .map(|block_and_id| block_and_id.height)
    }

    fn put_btc_block_in_db(&self, block: &BtcBlockInDbFormat) -> Result<()> {
        debug!("✔ Putting BTC block in db: {:?}", block);
        block
            .to_bytes()
            .and_then(|bytes| self.db.put(block.get_db_key(), bytes, MIN_DATA_SENSITIVITY_LEVEL))
    }

    fn put_special_btc_block_in_db(&self, block_and_id: &BtcBlockInDbFormat, block_type: &str) -> Result<()> {
        debug!("✔ Putting special BTC block in db of type: {}", block_type);
        self.put_btc_block_in_db(block_and_id)
            .and_then(|_| self.put_special_btc_hash_in_db(block_type, &block_and_id.id))
    }

    fn put_btc_hash_in_db(&self, key: &[Byte], hash: &BlockHash) -> Result<()> {
        self.db.put(key.to_vec(), hash.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
    }

    fn put_special_btc_hash_in_db(&self, hash_type: &str, hash: &BlockHash) -> Result<()> {
        let key = match hash_type {
            "tail" => Ok(self.btc_tail_block_hash_key.to_vec()),
            "canon" => Ok(self.btc_canon_block_hash_key.to_vec()),
            "anchor" => Ok(self.btc_anchor_block_hash_key.to_vec()),
            "latest" => Ok(self.btc_latest_block_hash_key.to_vec()),
            _ => Err(AppError::Custom(format!(
                "✘ Cannot store special BTC hash of type: {}!",
                hash_type
            ))),
        }?;
        self.put_btc_hash_in_db(&key, hash)
    }

    fn btc_block_exists_in_db(&self, btc_block_id: &BlockHash) -> bool {
        info!(
            "✔ Checking for existence of BTC block: {}",
            hex::encode(btc_block_id.to_vec())
        );
        self.key_exists_in_db(&btc_block_id.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
    }

    // FIXME This could be a more generic fxn no? (Across all chains?)
    fn key_exists_in_db(&self, key: &[Byte], sensitivity: DataSensitivity) -> bool {
        debug!("✔ Checking for existence of key: {}", hex::encode(key));
        self.db.get(key.to_vec(), sensitivity).is_ok()
    }

    fn put_btc_canon_to_tip_length_in_db(&self, btc_canon_to_tip_length: u64) -> Result<()> {
        self.db.put(
            self.btc_canon_to_tip_length_key.to_vec(),
            convert_u64_to_bytes(btc_canon_to_tip_length),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_btc_canon_to_tip_length_from_db(&self) -> Result<u64> {
        self.db
            .get(self.btc_canon_to_tip_length_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_u64(&bytes))
    }

    fn put_btc_private_key_in_db(&self, pk: &BtcPrivateKey) -> Result<()> {
        debug!("✔ Saving BTC private key into db...");
        pk.write_to_db(self.db, &self.btc_private_key_db_key.to_vec())
    }

    fn get_btc_private_key_from_db(&self) -> Result<BtcPrivateKey> {
        self.db
            .get(self.btc_private_key_db_key.to_vec(), MAX_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| BtcPrivateKey::from_slice(&bytes[..], self.get_btc_network_from_db()?))
    }

    fn put_btc_canon_block_in_db(&self, block: &BtcBlockInDbFormat) -> Result<()> {
        debug!("✔ Putting BTC canon block in db...");
        self.put_special_btc_block_in_db(block, "canon")
    }

    fn get_btc_anchor_block_from_db(&self) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC anchor block from db...");
        self.get_special_btc_block_from_db("anchor")
    }

    fn get_btc_tail_block_from_db(&self) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC tail block from db...");
        self.get_special_btc_block_from_db("tail")
    }

    fn get_btc_canon_block_from_db(&self) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC canon block from db...");
        self.get_special_btc_block_from_db("canon")
    }

    fn get_btc_anchor_block_hash_from_db(&self) -> Result<BlockHash> {
        debug!("✔ Getting BTC anchor block hash from db...");
        self.get_btc_hash_from_db(&self.btc_anchor_block_hash_key.to_vec())
    }

    fn put_btc_anchor_block_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC anchor block hash in db...");
        self.put_btc_hash_in_db(&self.btc_anchor_block_hash_key.to_vec(), hash)
    }

    fn put_btc_latest_block_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC latest block hash in db...");
        self.put_btc_hash_in_db(&self.btc_latest_block_hash_key.to_vec(), hash)
    }

    fn put_btc_tail_block_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC tail block hash in db...");
        self.put_btc_hash_in_db(&self.btc_tail_block_hash_key.to_vec(), hash)
    }

    fn put_btc_canon_block_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC canon block hash in db...");
        self.put_btc_hash_in_db(&self.btc_canon_block_hash_key.to_vec(), hash)
    }

    fn get_btc_linker_hash_from_db(&self) -> Result<BlockHash> {
        debug!("✔ Getting BTC linker hash from db...");
        self.get_btc_hash_from_db(&self.btc_linker_hash_key.to_vec())
    }

    fn put_btc_linker_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC linker hash in db...");
        self.put_btc_hash_in_db(&self.btc_linker_hash_key.to_vec(), hash)
    }

    fn maybe_get_parent_btc_block_and_id(&self, id: &BlockHash) -> Option<BtcBlockInDbFormat> {
        debug!("✔ Maybe getting BTC parent block for id: {}", id);
        self.maybe_get_nth_ancestor_btc_block_and_id(id, 1)
    }

    fn maybe_get_btc_block_from_db(&self, id: &BlockHash) -> Option<BtcBlockInDbFormat> {
        debug!("✔ Maybe getting BTC block of id: {}", hex::encode(id.to_vec()));
        match self.get_btc_block_from_db(id) {
            Ok(block_and_id) => {
                debug!("✔ BTC block found!");
                Some(block_and_id)
            },
            Err(e) => {
                debug!("✘ No BTC block found ∵ {}", e);
                None
            },
        }
    }

    fn maybe_get_nth_ancestor_btc_block_and_id(&self, id: &BlockHash, n: u64) -> Option<BtcBlockInDbFormat> {
        debug!(
            "✔ Maybe getting ancestor #{} of BTC block id: {}",
            n,
            hex::encode(id.to_vec())
        );
        match self.maybe_get_btc_block_from_db(id) {
            None => {
                debug!("✘ No ancestor #{} BTC block found!", n);
                None
            },
            Some(block_in_db_format) => match n {
                0 => Some(block_in_db_format),
                _ => self.maybe_get_nth_ancestor_btc_block_and_id(&block_in_db_format.prev_blockhash, n - 1),
            },
        }
    }

    fn put_btc_address_in_db(&self, btc_address: &str) -> Result<()> {
        debug!("✔ Putting BTC address {} in db...", btc_address);
        self.db.put(
            self.btc_address_key.to_vec(),
            convert_btc_address_to_bytes(btc_address)?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn get_btc_address_from_db(&self) -> Result<String> {
        debug!("✔  Getting BTC address from db...");
        self.db
            .get(self.btc_address_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .map(convert_bytes_to_btc_address)
    }
}

fn start_btc_db_transaction<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    state.db.start_transaction().map(|_| {
        info!("✔ Database transaction begun forj BTC block submission!");
        state
    })
}

fn end_btc_db_transaction<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    state.db.end_transaction().map(|_| {
        info!("✔ Database transaction ended for BTC block submission!");
        state
    })
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::btc::btc_test_utils::{
            get_sample_btc_block_in_db_format,
            get_sample_btc_private_key,
            get_sample_btc_pub_key_slice,
            get_sample_sequential_btc_blocks_in_db_format,
            SAMPLE_TARGET_BTC_ADDRESS,
        },
        test_utils::get_test_database,
    };

    #[test]
    fn non_existing_key_should_not_exist_in_db() {
        let db = get_test_database();
        let result = key_exists_in_db(&db, &BTC_CANON_TO_TIP_LENGTH_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL);
        assert!(!result);
    }

    #[test]
    fn existing_key_should_exist_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let length = 5;
        put_btc_canon_to_tip_length_in_db(&db, length).unwrap();
        let result = key_exists_in_db(&db, &BTC_CANON_TO_TIP_LENGTH_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL);
        assert!(result);
    }

    #[test]
    fn should_get_and_put_btc_canon_to_tip_length_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let length = 6;
        put_btc_canon_to_tip_length_in_db(&db, length).unwrap();
        let result = get_btc_canon_to_tip_length_from_db(&db).unwrap();
        assert_eq!(result, length);
    }

    #[test]
    fn should_get_and_save_btc_private_key_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        put_btc_network_in_db(&db, BtcNetwork::Testnet).unwrap();
        let pk = get_sample_btc_private_key();
        put_btc_private_key_in_db(&db, &pk).unwrap();
        let result = get_btc_private_key_from_db(&db).unwrap();
        assert_eq!(result.to_public_key(), pk.to_public_key());
    }

    #[test]
    fn should_error_putting_non_existent_block_type_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let non_existent_block_type = "non-existent block type!";
        let block = get_sample_btc_block_in_db_format().unwrap();
        let expected_error = format!("✘ Cannot store special BTC hash of type: {}!", non_existent_block_type);
        match put_special_btc_block_in_db(&db, &block, non_existent_block_type) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Should not have succeeded!"),
            _ => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_put_special_block_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        let block_type = "canon";
        put_special_btc_block_in_db(&db, &block, block_type).unwrap();
        match get_btc_canon_block_from_db(&db) {
            Err(e) => panic!("Error geting canon block: {}", e),
            Ok(block_from_db) => assert_eq!(block_from_db, block),
        }
    }

    #[test]
    fn should_error_getting_non_existent_special_block() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let non_existent_block_type = "does not exist";
        let expected_error = format!("✘ Cannot get special BTC hash of type: {}!", non_existent_block_type);
        match get_special_btc_block_from_db(&db, non_existent_block_type) {
            Ok(_) => panic!("Should not have got special block!"),
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            _ => panic!("Wrong error when getting non-existent block type!"),
        }
    }

    #[test]
    fn should_get_special_block_type() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        put_btc_block_in_db(&db, &block).unwrap();
        put_btc_anchor_block_hash_in_db(&db, &block.id).unwrap();
        let result = get_special_btc_block_from_db(&db, "anchor").unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn should_get_and_put_anchor_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let anchor_block_hash = get_sample_btc_block_in_db_format().unwrap().id;
        put_btc_anchor_block_hash_in_db(&db, &anchor_block_hash).unwrap();
        let result = get_btc_anchor_block_hash_from_db(&db).unwrap();
        assert_eq!(result, anchor_block_hash);
    }

    #[test]
    fn should_put_latest_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let latest_block_hash = get_sample_btc_block_in_db_format().unwrap().id;
        put_btc_latest_block_hash_in_db(&db, &latest_block_hash).unwrap();
    }

    #[test]
    fn should_put_canon_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let canon_block_hash = get_sample_btc_block_in_db_format().unwrap().id;
        put_btc_canon_block_hash_in_db(&db, &canon_block_hash).unwrap();
    }

    #[test]
    fn should_get_and_put_linker_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let linker_hash = get_sample_btc_block_in_db_format().unwrap().id;
        put_btc_linker_hash_in_db(&db, &linker_hash).unwrap();
        let result = get_btc_linker_hash_from_db(&db).unwrap();
        assert_eq!(result, linker_hash);
    }

    #[test]
    fn should_put_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let hash = get_sample_btc_block_in_db_format().unwrap().id;
        put_btc_hash_in_db(&db, &BTC_LINKER_HASH_KEY.to_vec(), &hash).unwrap();
        let result = get_btc_hash_from_db(&db, &BTC_LINKER_HASH_KEY.to_vec()).unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_not_get_parent_block_if_non_existent() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let test_block = get_sample_btc_block_in_db_format().unwrap();
        assert!(maybe_get_parent_btc_block_and_id(&db, &test_block.id).is_none());
    }

    #[test]
    fn should_get_parent_block() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let blocks = get_sample_sequential_btc_blocks_in_db_format();
        let test_block = blocks[blocks.len() - 1].clone();
        let expected_result = blocks[blocks.len() - 2].clone();
        blocks
            .iter()
            .map(|block| put_btc_block_in_db(&db, &block))
            .collect::<Result<()>>()
            .unwrap();
        let result = maybe_get_parent_btc_block_and_id(&db, &test_block.id).unwrap();
        assert_eq!(result, expected_result);
        assert!(result.id == test_block.prev_blockhash);
    }

    #[test]
    fn should_get_and_put_btc_block_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let block_and_id = get_sample_btc_block_in_db_format().unwrap();
        put_btc_block_in_db(&db, &block_and_id).unwrap();
        let result = get_btc_block_from_db(&db, &block_and_id.id).unwrap();
        assert_eq!(result, block_and_id);
    }

    #[test]
    fn should_get_and_put_btc_address_in_database() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        put_btc_address_in_db(&db, &SAMPLE_TARGET_BTC_ADDRESS.to_string()).unwrap();
        let result = get_btc_address_from_db(&db).unwrap();
        assert_eq!(result, SAMPLE_TARGET_BTC_ADDRESS);
    }

    #[test]
    fn should_get_and_put_btc_fee_in_db() {
        let fee = 666;
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        put_btc_fee_in_db(&db, fee).unwrap();
        let result = get_btc_fee_from_db(&db).unwrap();
        assert_eq!(result, fee)
    }

    #[test]
    fn should_get_and_put_btc_network_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let network = BtcNetwork::Bitcoin;
        put_btc_network_in_db(&db, network).unwrap();
        let result = get_btc_network_from_db(&db).unwrap();
        assert_eq!(result, network)
    }

    #[test]
    fn should_get_and_put_btc_difficulty_in_db() {
        let difficulty = 1337;
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        put_btc_difficulty_in_db(&db, difficulty).unwrap();
        let result = get_btc_difficulty_from_db(&db).unwrap();
        assert_eq!(result, difficulty)
    }

    #[test]
    fn should_maybe_get_btc_block_from_db_if_none_extant() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        let block_hash = block.id;
        assert!(maybe_get_btc_block_from_db(&db, &block_hash).is_none());
    }

    #[test]
    fn should_maybe_get_btc_block_from_db_if_extant() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        put_btc_block_in_db(&db, &block).unwrap();
        let block_hash = block.id;
        let result = maybe_get_btc_block_from_db(&db, &block_hash).unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn none_existent_block_should_not_exist_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let block_hash = get_sample_btc_block_in_db_format().unwrap().id;
        let result = btc_block_exists_in_db(&db, &block_hash);
        assert!(!result);
    }

    #[test]
    fn existing_block_should_exist_in_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        put_btc_block_in_db(&db, &block).unwrap();
        let block_hash = block.id;
        let result = btc_block_exists_in_db(&db, &block_hash);
        assert!(result);
    }

    #[test]
    fn should_save_and_get_btc_pub_key_slice_from_db() {
        let db = get_test_database();
        let db_utils = BtcDatabaseUtils::new(&db);
        let pub_key_slice = get_sample_btc_pub_key_slice();
        put_btc_pub_key_slice_in_db(&db, &pub_key_slice).unwrap();
        let result = get_btc_public_key_slice_from_db(&db).unwrap();
        result
            .iter()
            .enumerate()
            .for_each(|(i, e)| assert_eq!(e, &pub_key_slice[i]));
    }
}
*/
