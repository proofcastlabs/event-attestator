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

create_db_utils!(
    "Btc";
    "_FEE_KEY" => "btc-fee-key",
    "_ADDRESS_KEY" => "btc-address",
    "_DIFFICULTY" => "btc-difficulty",
    "_NETWORK_KEY" => "btc-network-key",
    "_LINKER_HASH_KEY" => "btc-linker-hash",
    "_PRIVATE_KEY_DB_KEY" => "btc-private-key",
    "_CANON_BLOCK_HASH_KEY" => "btc-canon-block",
    "_LATEST_BLOCK_HASH_KEY" => "btc-latest-block",
    "_ANCHOR_BLOCK_HASH_KEY" => "btc-anchor-block",
    "_PTOKEN_GENESIS_HASH_KEY" => "provable-ptoken",
    "_ACCOUNT_NONCE_KEY" => "btc-account-nonce-key",
    "_PUBLIC_KEY_DB_KEY" => "btc-public-key-db-key",
    "_TAIL_BLOCK_HASH_KEY" => "btc-tail-block-hash-key",
    "_CANON_TO_TIP_LENGTH_KEY" => "btc-canon-to-tip-length"
);

impl<'a, D: DatabaseInterface>BtcDbUtils<'a, D> {
    pub fn get_db(&self) -> &D {
        self.db
    }

    pub fn get_btc_chain_id_from_db(&self) -> Result<BtcChainId> {
        self.db
            .get(self.btc_network_key.clone(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|ref bytes| BtcChainId::from_bytes(bytes))
    }

    pub fn put_btc_pub_key_slice_in_db(&self, pub_key_slice: &BtcPubKeySlice) -> Result<()> {
        self.db.put(
            self.btc_public_key_db_key.clone(),
            pub_key_slice.to_vec(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_btc_public_key_slice_from_db(&self) -> Result<BtcPubKeySlice> {
        self.db
            .get(self.btc_public_key_db_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_btc_pub_key_slice(&bytes))
    }

    pub fn increment_btc_account_nonce_in_db(&self, amount_to_increment_by: u64) -> Result<()> {
        info!("✔ Incrementing BTC account nonce in db...");
        self.get_btc_account_nonce_from_db()
            .and_then(|nonce| self.put_btc_account_nonce_in_db(nonce + amount_to_increment_by))
    }

    pub fn put_btc_account_nonce_in_db(&self, nonce: u64) -> Result<()> {
        debug!("✔ Putting BTC account nonce of {} in db...", nonce);
        put_u64_in_db(self.db, &self.btc_account_nonce_key.to_vec(), nonce)
    }

    pub fn get_btc_account_nonce_from_db(&self) -> Result<u64> {
        debug!("✔ Getting BTC account nonce from db...");
        get_u64_from_db(self.db, &self.btc_account_nonce_key.to_vec())
    }

    pub fn get_btc_fee_from_db(&self) -> Result<u64> {
        debug!("✔ Getting BTC fee from db...");
        self.db
            .get(self.btc_fee_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_u64(&bytes))
    }

    pub fn put_btc_fee_in_db(&self, fee: u64) -> Result<()> {
        // FIXME should not be allowed to change once set!
        debug!("✔ Adding BTC fee of '{}' satoshis-per-byte to db...", fee);
        self.db.put(
            self.btc_fee_key.to_vec(),
            convert_u64_to_bytes(fee),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_btc_network_from_db(&self) -> Result<BtcNetwork> {
        self.db
            .get(self.btc_network_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|ref bytes| BtcChainId::from_bytes(bytes))
            .map(|chain_id| chain_id.to_btc_network())
    }

    pub fn put_btc_network_in_db(&self, network: BtcNetwork) -> Result<()> {
        // FIXME should not be allowed to change once set!
        info!("✔ Adding BTC '{}' network to database...", network);
        self.db.put(
            self.btc_network_key.to_vec(),
            BtcChainId::from_btc_network(&network)?.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn put_btc_difficulty_in_db(&self, difficulty: u64) -> Result<()> {
        debug!("✔ Putting BTC difficulty threshold of {} in db...", difficulty);
        self.db.put(
            self.btc_difficulty.to_vec(),
            convert_u64_to_bytes(difficulty),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_btc_difficulty_from_db(&self) -> Result<u64> {
        debug!("✔ Getting BTC difficulty threshold from db...");
        self.db
            .get(self.btc_difficulty.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_u64(&bytes))
    }

    pub fn get_btc_block_from_db(&self, id: &BlockHash) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC block from db via id: {}", hex::encode(id));
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

    pub fn get_btc_latest_block_from_db(&self) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC latest block from db...");
        self.get_special_btc_block_from_db("latest")
    }

    pub fn get_latest_btc_block_number(&self) -> Result<u64> {
        debug!("✔ Getting BTC latest block number from db...");
        self.get_btc_latest_block_from_db()
            .map(|block_and_id| block_and_id.height)
    }

    pub fn put_btc_block_in_db(&self, block: &BtcBlockInDbFormat) -> Result<()> {
        debug!("✔ Putting BTC block in db: {:?}", block);
        block
            .to_bytes()
            .and_then(|bytes| self.db.put(block.get_db_key(), bytes, MIN_DATA_SENSITIVITY_LEVEL))
    }

    pub fn put_special_btc_block_in_db(&self, block_and_id: &BtcBlockInDbFormat, block_type: &str) -> Result<()> {
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

    pub fn btc_block_exists_in_db(&self, btc_block_id: &BlockHash) -> bool {
        info!("✔ Checking for existence of BTC block: {}", hex::encode(btc_block_id));
        self.key_exists_in_db(btc_block_id, MIN_DATA_SENSITIVITY_LEVEL)
    }

    // FIXME This could be a more generic fxn no? (Across all chains?)
    pub fn key_exists_in_db(&self, key: &[Byte], sensitivity: DataSensitivity) -> bool {
        debug!("✔ Checking for existence of key: {}", hex::encode(key));
        self.db.get(key.to_vec(), sensitivity).is_ok()
    }

    pub fn put_btc_canon_to_tip_length_in_db(&self, btc_canon_to_tip_length: u64) -> Result<()> {
        self.db.put(
            self.btc_canon_to_tip_length_key.to_vec(),
            convert_u64_to_bytes(btc_canon_to_tip_length),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_btc_canon_to_tip_length_from_db(&self) -> Result<u64> {
        self.db
            .get(self.btc_canon_to_tip_length_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| convert_bytes_to_u64(&bytes))
    }

    pub fn put_btc_private_key_in_db(&self, pk: &BtcPrivateKey) -> Result<()> {
        // FIXME Should not be allowed to change once set.
        debug!("✔ Saving BTC private key into db...");
        pk.write_to_db(self.db, &self.btc_private_key_db_key.to_vec())
    }

    pub fn get_btc_private_key_from_db(&self) -> Result<BtcPrivateKey> {
        self.db
            .get(self.btc_private_key_db_key.to_vec(), MAX_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| BtcPrivateKey::from_slice(&bytes[..], self.get_btc_network_from_db()?))
    }

    pub fn put_btc_canon_block_in_db(&self, block: &BtcBlockInDbFormat) -> Result<()> {
        debug!("✔ Putting BTC canon block in db...");
        self.put_special_btc_block_in_db(block, "canon")
    }

    pub fn get_btc_anchor_block_from_db(&self) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC anchor block from db...");
        self.get_special_btc_block_from_db("anchor")
    }

    pub fn get_btc_tail_block_from_db(&self) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC tail block from db...");
        self.get_special_btc_block_from_db("tail")
    }

    pub fn get_btc_canon_block_from_db(&self) -> Result<BtcBlockInDbFormat> {
        debug!("✔ Getting BTC canon block from db...");
        self.get_special_btc_block_from_db("canon")
    }

    pub fn get_btc_anchor_block_hash_from_db(&self) -> Result<BlockHash> {
        debug!("✔ Getting BTC anchor block hash from db...");
        self.get_btc_hash_from_db(&self.btc_anchor_block_hash_key.to_vec())
    }

    pub fn put_btc_anchor_block_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC anchor block hash in db...");
        self.put_btc_hash_in_db(&self.btc_anchor_block_hash_key.to_vec(), hash)
    }

    pub fn put_btc_latest_block_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC latest block hash in db...");
        self.put_btc_hash_in_db(&self.btc_latest_block_hash_key.to_vec(), hash)
    }

    pub fn put_btc_tail_block_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC tail block hash in db...");
        self.put_btc_hash_in_db(&self.btc_tail_block_hash_key.to_vec(), hash)
    }

    pub fn put_btc_canon_block_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC canon block hash in db...");
        self.put_btc_hash_in_db(&self.btc_canon_block_hash_key.to_vec(), hash)
    }

    pub fn get_btc_linker_hash_from_db(&self) -> Result<BlockHash> {
        debug!("✔ Getting BTC linker hash from db...");
        self.get_btc_hash_from_db(&self.btc_linker_hash_key.to_vec())
    }

    pub fn put_btc_linker_hash_in_db(&self, hash: &BlockHash) -> Result<()> {
        debug!("✔ Putting BTC linker hash in db...");
        self.put_btc_hash_in_db(&self.btc_linker_hash_key.to_vec(), hash)
    }

    pub fn maybe_get_parent_btc_block_and_id(&self, id: &BlockHash) -> Option<BtcBlockInDbFormat> {
        debug!("✔ Maybe getting BTC parent block for id: {}", id);
        self.maybe_get_nth_ancestor_btc_block_and_id(id, 1)
    }

    fn maybe_get_btc_block_from_db(&self, id: &BlockHash) -> Option<BtcBlockInDbFormat> {
        debug!("✔ Maybe getting BTC block of id: {}", hex::encode(id));
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

    pub fn maybe_get_nth_ancestor_btc_block_and_id(&self, id: &BlockHash, n: u64) -> Option<BtcBlockInDbFormat> {
        debug!("✔ Maybe getting ancestor #{} of BTC block id: {}", n, hex::encode(id));
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

    pub fn put_btc_address_in_db(&self, btc_address: &str) -> Result<()> {
        // FIXME should not be allowed to change once set!
        debug!("✔ Putting BTC address {} in db...", btc_address);
        self.db.put(
            self.btc_address_key.to_vec(),
            convert_btc_address_to_bytes(btc_address)?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_btc_address_from_db(&self) -> Result<String> {
        debug!("✔  Getting BTC address from db...");
        self.db
            .get(self.btc_address_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .map(convert_bytes_to_btc_address)
    }
}

pub fn start_btc_db_transaction<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    state.db.start_transaction().map(|_| {
        info!("✔ Database transaction begun forj BTC block submission!");
        state
    })
}

pub fn end_btc_db_transaction<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    state.db.end_transaction().map(|_| {
        info!("✔ Database transaction ended for BTC block submission!");
        state
    })
}

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
        let db_utils = BtcDbUtils::new(&db);
        let result = db_utils.key_exists_in_db(&BTC_CANON_TO_TIP_LENGTH_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL);
        assert!(!result);
    }

    #[test]
    fn existing_key_should_exist_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let length = 5;
        db_utils.put_btc_canon_to_tip_length_in_db(length).unwrap();
        let result = db_utils.key_exists_in_db(&BTC_CANON_TO_TIP_LENGTH_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL);
        assert!(result);
    }

    #[test]
    fn should_get_and_put_btc_canon_to_tip_length_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let length = 6;
        db_utils.put_btc_canon_to_tip_length_in_db(length).unwrap();
        let result = db_utils.get_btc_canon_to_tip_length_from_db().unwrap();
        assert_eq!(result, length);
    }

    #[test]
    fn should_get_and_save_btc_private_key_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        db_utils.put_btc_network_in_db(BtcNetwork::Testnet).unwrap();
        let pk = get_sample_btc_private_key();
        db_utils.put_btc_private_key_in_db(&pk).unwrap();
        let result = db_utils.get_btc_private_key_from_db().unwrap();
        assert_eq!(result.to_public_key(), pk.to_public_key());
    }

    #[test]
    fn should_error_putting_non_existent_block_type_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let non_existent_block_type = "non-existent block type!";
        let block = get_sample_btc_block_in_db_format().unwrap();
        let expected_error = format!("✘ Cannot store special BTC hash of type: {}!", non_existent_block_type);
        match db_utils.put_special_btc_block_in_db(&block, non_existent_block_type) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Should not have succeeded!"),
            _ => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_put_special_block_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        let block_type = "canon";
        db_utils.put_special_btc_block_in_db(&block, block_type).unwrap();
        match db_utils.get_btc_canon_block_from_db() {
            Err(e) => panic!("Error geting canon block: {}", e),
            Ok(block_from_db) => assert_eq!(block_from_db, block),
        }
    }

    #[test]
    fn should_error_getting_non_existent_special_block() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let non_existent_block_type = "does not exist";
        let expected_error = format!("✘ Cannot get special BTC hash of type: {}!", non_existent_block_type);
        match db_utils.get_special_btc_block_from_db(non_existent_block_type) {
            Ok(_) => panic!("Should not have got special block!"),
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            _ => panic!("Wrong error when getting non-existent block type!"),
        }
    }

    #[test]
    fn should_get_special_block_type() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        db_utils.put_btc_block_in_db(&block).unwrap();
        db_utils.put_btc_anchor_block_hash_in_db(&block.id).unwrap();
        let result = db_utils.get_special_btc_block_from_db("anchor").unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn should_get_and_put_anchor_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let anchor_block_hash = get_sample_btc_block_in_db_format().unwrap().id;
        db_utils.put_btc_anchor_block_hash_in_db(&anchor_block_hash).unwrap();
        let result = db_utils.get_btc_anchor_block_hash_from_db().unwrap();
        assert_eq!(result, anchor_block_hash);
    }

    #[test]
    fn should_put_latest_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let latest_block_hash = get_sample_btc_block_in_db_format().unwrap().id;
        db_utils.put_btc_latest_block_hash_in_db(&latest_block_hash).unwrap();
    }

    #[test]
    fn should_put_canon_block_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let canon_block_hash = get_sample_btc_block_in_db_format().unwrap().id;
        db_utils.put_btc_canon_block_hash_in_db(&canon_block_hash).unwrap();
    }

    #[test]
    fn should_get_and_put_linker_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let linker_hash = get_sample_btc_block_in_db_format().unwrap().id;
        db_utils.put_btc_linker_hash_in_db(&linker_hash).unwrap();
        let result = db_utils.get_btc_linker_hash_from_db().unwrap();
        assert_eq!(result, linker_hash);
    }

    #[test]
    fn should_put_hash_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let hash = get_sample_btc_block_in_db_format().unwrap().id;
        db_utils
            .put_btc_hash_in_db(&BTC_LINKER_HASH_KEY.to_vec(), &hash)
            .unwrap();
        let result = db_utils.get_btc_hash_from_db(&BTC_LINKER_HASH_KEY.to_vec()).unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn should_not_get_parent_block_if_non_existent() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let test_block = get_sample_btc_block_in_db_format().unwrap();
        let result = db_utils.maybe_get_parent_btc_block_and_id(&test_block.id);
        assert!(result.is_none());
    }

    #[test]
    fn should_get_parent_block() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let blocks = get_sample_sequential_btc_blocks_in_db_format();
        let test_block = blocks[blocks.len() - 1].clone();
        let expected_result = blocks[blocks.len() - 2].clone();
        blocks
            .iter()
            .map(|block| db_utils.put_btc_block_in_db(&block))
            .collect::<Result<()>>()
            .unwrap();
        let result = db_utils.maybe_get_parent_btc_block_and_id(&test_block.id).unwrap();
        assert_eq!(result, expected_result);
        assert!(result.id == test_block.prev_blockhash);
    }

    #[test]
    fn should_get_and_put_btc_block_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let block_and_id = get_sample_btc_block_in_db_format().unwrap();
        db_utils.put_btc_block_in_db(&block_and_id).unwrap();
        let result = db_utils.get_btc_block_from_db(&block_and_id.id).unwrap();
        assert_eq!(result, block_and_id);
    }

    #[test]
    fn should_get_and_put_btc_address_in_database() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        db_utils
            .put_btc_address_in_db(&SAMPLE_TARGET_BTC_ADDRESS.to_string())
            .unwrap();
        let result = db_utils.get_btc_address_from_db().unwrap();
        assert_eq!(result, SAMPLE_TARGET_BTC_ADDRESS);
    }

    #[test]
    fn should_get_and_put_btc_fee_in_db() {
        let fee = 666;
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        db_utils.put_btc_fee_in_db(fee).unwrap();
        let result = db_utils.get_btc_fee_from_db().unwrap();
        assert_eq!(result, fee)
    }

    #[test]
    fn should_get_and_put_btc_network_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let network = BtcNetwork::Bitcoin;
        db_utils.put_btc_network_in_db(network).unwrap();
        let result = db_utils.get_btc_network_from_db().unwrap();
        assert_eq!(result, network)
    }

    #[test]
    fn should_get_and_put_btc_difficulty_in_db() {
        let difficulty = 1337;
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        db_utils.put_btc_difficulty_in_db(difficulty).unwrap();
        let result = db_utils.get_btc_difficulty_from_db().unwrap();
        assert_eq!(result, difficulty)
    }

    #[test]
    fn should_maybe_get_btc_block_from_db_if_none_extant() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        let block_hash = block.id;
        let result = db_utils.maybe_get_btc_block_from_db(&block_hash);
        assert!(result.is_none());
    }

    #[test]
    fn should_maybe_get_btc_block_from_db_if_extant() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        db_utils.put_btc_block_in_db(&block).unwrap();
        let block_hash = block.id;
        let result = db_utils.maybe_get_btc_block_from_db(&block_hash).unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn none_existent_block_should_not_exist_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let block_hash = get_sample_btc_block_in_db_format().unwrap().id;
        let result = db_utils.btc_block_exists_in_db(&block_hash);
        assert!(!result);
    }

    #[test]
    fn existing_block_should_exist_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let block = get_sample_btc_block_in_db_format().unwrap();
        db_utils.put_btc_block_in_db(&block).unwrap();
        let block_hash = block.id;
        let result = db_utils.btc_block_exists_in_db(&block_hash);
        assert!(result);
    }

    #[test]
    fn should_save_and_get_btc_pub_key_slice_from_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let pub_key_slice = get_sample_btc_pub_key_slice();
        db_utils.put_btc_pub_key_slice_in_db(&pub_key_slice).unwrap();
        let result = db_utils.get_btc_public_key_slice_from_db().unwrap();
        result
            .iter()
            .enumerate()
            .for_each(|(i, e)| assert_eq!(e, &pub_key_slice[i]));
    }

    #[test]
    fn btc_db_keys_should_stay_consistent() {
        #[rustfmt::skip]
        let expected_result = BtcDatabaseKeysJson {
            BTC_ACCOUNT_NONCE_KEY:
                "48236d034b7d7fac3b4550bdbe5682eb012d1717bb345c39c5add04be5139880".to_string(),
            BTC_ADDRESS_KEY:
                "bdf6e75595f2a65ce048e0416b8c2a8462288116db886b551b2891adceb0a53a".to_string(),
            BTC_ANCHOR_BLOCK_HASH_KEY:
                "bb005e5d49d23fc16c62b7971672f0f44043866cf19e4aa2d77db7f9632d0d83".to_string(),
            BTC_CANON_BLOCK_HASH_KEY:
                "ed228247ba940027aa9406ef39c2aa07f650bfa53f0b8478f2d90836615912b8".to_string(),
            BTC_CANON_TO_TIP_LENGTH_KEY:
                "2d9b6327983926c2dd9636f3c8bc13b811af80858c08fe1b9d019ebdcf73049c".to_string(),
            BTC_DIFFICULTY:
                "0ed532c16cd0bcc543cdcd01132c38349fd25e85b2d7f4609b66943bc8500a7c".to_string(),
            BTC_FEE_KEY:
                "6ded8f6cf1097edaf81e815dec1810946dd32327ecdc9de506ca7d1535c34801".to_string(),
            BTC_LATEST_BLOCK_HASH_KEY:
                "22f781fdf51ac53605f603b9abeaddd618d29eb7ebed285a919abf128379a0a2".to_string(),
            BTC_LINKER_HASH_KEY:
                "98e63aa8f93943b3bfea2ee4d0e063942415618cfc0cd51828de4de7b4698039".to_string(),
            BTC_NETWORK_KEY:
                "f2321e29a0792487edd90debfc9a85fcb39856a5343801e794c5c915aa341ee8".to_string(),
            BTC_PRIVATE_KEY_DB_KEY:
                "d8c4da823c79e9245163a8db18b7e9d6107f7487e624a4db9bdc3acb788902de".to_string(),
            BTC_PUBLIC_KEY_DB_KEY:
                "ee7ec6657db53cd1d8055d61bf00ff615063701493ede450dc5c31132ae6cfd1".to_string(),
            BTC_TAIL_BLOCK_HASH_KEY:
                "26ab99d609131225d7ecf087632b5b6771468931273d0f6c16b09c9bbe316f71".to_string(),
            BTC_PTOKEN_GENESIS_HASH_KEY:
                "7eb2e65416dd107602495454d1ed094ae475cff2f3bfb2e2ae68a1c52bc0d66f".to_string(),
        };
        let result = BtcDatabaseKeysJson::new();
        assert_eq!(result, expected_result);
    }
}
