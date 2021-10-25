use ethereum_types::{Address as EthAddress, H256 as EthHash};

use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_crypto::eth_private_key::EthPrivateKey,
        eth_submission_material::EthSubmissionMaterial,
        evm_constants::{
            EVM_ACCOUNT_NONCE_KEY,
            EVM_ADDRESS_KEY,
            EVM_ANCHOR_BLOCK_HASH_KEY,
            EVM_ANY_SENDER_NONCE_KEY,
            EVM_CANON_BLOCK_HASH_KEY,
            EVM_CANON_TO_TIP_LENGTH_KEY,
            EVM_CHAIN_ID_KEY,
            EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY,
            EVM_GAS_PRICE_KEY,
            EVM_LATEST_BLOCK_HASH_KEY,
            EVM_LINKER_HASH_KEY,
            EVM_PRIVATE_KEY_DB_KEY,
            EVM_TAIL_BLOCK_HASH_KEY,
        },
    },
    constants::{MAX_DATA_SENSITIVITY_LEVEL, MIN_DATA_SENSITIVITY_LEVEL},
    database_utils::{get_u64_from_db, put_u64_in_db},
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, DataSensitivity, Result},
    utils::{convert_bytes_to_u64, convert_u64_to_bytes},
};

fn convert_h256_to_bytes(hash: &EthHash) -> Bytes {
    // NOTE: We switch the endianness of the block hash to avoid DB collisions w/ ETH<->ETH bridges.
    let mut bytes = hash.as_bytes().to_vec();
    bytes.reverse();
    bytes
}

pub fn delete_block_by_block_hash<D: DatabaseInterface>(db: &D, block_hash: &EthHash) -> Result<()> {
    db.delete(convert_h256_to_bytes(block_hash))
}

pub fn put_eth_canon_to_tip_length_in_db<D: DatabaseInterface>(db: &D, length: u64) -> Result<()> {
    debug!("✔ Putting ETH canon-to-tip length of {} in db...", length);
    db.put(
        EVM_CANON_TO_TIP_LENGTH_KEY.to_vec(),
        convert_u64_to_bytes(length),
        MIN_DATA_SENSITIVITY_LEVEL,
    )
}

pub fn get_eth_canon_to_tip_length_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    info!("✔ Getting ETH canon-to-tip length from db...");
    db.get(EVM_CANON_TO_TIP_LENGTH_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
        .and_then(|bytes| convert_bytes_to_u64(&bytes))
}

pub fn put_eth_canon_block_in_db<D: DatabaseInterface>(
    db: &D,
    eth_submission_material: &EthSubmissionMaterial,
) -> Result<()> {
    info!("✔ Putting ETH canon block in db...");
    put_special_eth_block_in_db(db, eth_submission_material, "canon")
}

#[cfg(test)]
pub fn put_eth_latest_block_in_db<D: DatabaseInterface>(
    db: &D,
    eth_submission_material: &EthSubmissionMaterial,
) -> Result<()> {
    info!("✔ Putting ETH latest block in db...");
    put_special_eth_block_in_db(db, eth_submission_material, "latest")
}

pub fn put_eth_latest_block_hash_in_db<D: DatabaseInterface>(db: &D, eth_hash: &EthHash) -> Result<()> {
    info!("✔ Putting ETH latest block hash in db...");
    put_special_eth_hash_in_db(db, "latest", eth_hash)
}

pub fn put_eth_anchor_block_hash_in_db<D: DatabaseInterface>(db: &D, eth_hash: &EthHash) -> Result<()> {
    info!("✔ Putting ETH anchor block hash in db...");
    put_special_eth_hash_in_db(db, "anchor", eth_hash)
}

pub fn put_eth_canon_block_hash_in_db<D: DatabaseInterface>(db: &D, eth_hash: &EthHash) -> Result<()> {
    info!("✔ Putting ETH canon block hash in db...");
    put_special_eth_hash_in_db(db, "canon", eth_hash)
}

pub fn put_eth_tail_block_hash_in_db<D: DatabaseInterface>(db: &D, eth_hash: &EthHash) -> Result<()> {
    info!("✔ Putting ETH tail block hash in db...");
    put_special_eth_hash_in_db(db, "tail", eth_hash)
}

pub fn put_eth_linker_hash_in_db<D: DatabaseInterface>(db: &D, eth_hash: EthHash) -> Result<()> {
    info!("✔ Putting ETH linker hash in db...");
    put_special_eth_hash_in_db(db, "linker", &eth_hash)
}

pub fn get_eth_linker_hash_from_db<D: DatabaseInterface>(db: &D) -> Result<EthHash> {
    info!("✔ Getting ETH linker hash in db...");
    get_special_eth_hash_from_db(db, "linker")
}

pub fn put_special_eth_block_in_db<D: DatabaseInterface>(
    db: &D,
    eth_submission_material: &EthSubmissionMaterial,
    block_type: &str,
) -> Result<()> {
    trace!("✔ Putting ETH special block in db of type: {}", block_type);
    put_eth_submission_material_in_db(db, eth_submission_material)
        .and_then(|_| put_special_eth_hash_in_db(db, block_type, &eth_submission_material.get_block_hash()?))
}

pub fn put_special_eth_hash_in_db<D: DatabaseInterface>(db: &D, hash_type: &str, hash: &EthHash) -> Result<()> {
    let key = match hash_type {
        "linker" => Ok(EVM_LINKER_HASH_KEY.to_vec()),
        "canon" => Ok(EVM_CANON_BLOCK_HASH_KEY.to_vec()),
        "tail" => Ok(EVM_TAIL_BLOCK_HASH_KEY.to_vec()),
        "anchor" => Ok(EVM_ANCHOR_BLOCK_HASH_KEY.to_vec()),
        "latest" => Ok(EVM_LATEST_BLOCK_HASH_KEY.to_vec()),
        _ => Err(AppError::Custom(format!(
            "✘ Cannot store special ETH hash of type: {}!",
            hash_type
        ))),
    }?;
    put_eth_hash_in_db(db, &key, hash)
}

pub fn get_latest_eth_block_number<D: DatabaseInterface>(db: &D) -> Result<usize> {
    info!("✔ Getting latest ETH block number from db...");
    match get_special_eth_block_from_db(db, "latest") {
        Ok(result) => Ok(result.get_block_number()?.as_usize()),
        Err(e) => Err(e),
    }
}

pub fn get_eth_tail_block_from_db<D: DatabaseInterface>(db: &D) -> Result<EthSubmissionMaterial> {
    info!("✔ Getting ETH tail block from db...");
    get_special_eth_block_from_db(db, "tail")
}

pub fn get_eth_latest_block_from_db<D: DatabaseInterface>(db: &D) -> Result<EthSubmissionMaterial> {
    info!("✔ Getting ETH latest block from db...");
    get_special_eth_block_from_db(db, "latest")
}

pub fn get_eth_anchor_block_from_db<D: DatabaseInterface>(db: &D) -> Result<EthSubmissionMaterial> {
    info!("✔ Getting ETH anchor block from db...");
    get_special_eth_block_from_db(db, "anchor")
}

pub fn get_eth_canon_block_from_db<D: DatabaseInterface>(db: &D) -> Result<EthSubmissionMaterial> {
    info!("✔ Getting ETH canon block from db...");
    get_special_eth_block_from_db(db, "canon")
}

pub fn get_eth_anchor_block_hash_from_db<D: DatabaseInterface>(db: &D) -> Result<EthHash> {
    info!("✔ Getting ETH anchor block hash from db...");
    get_special_eth_hash_from_db(db, "anchor")
}

#[cfg(test)]
pub fn get_eth_latest_block_hash_from_db<D: DatabaseInterface>(db: &D) -> Result<EthHash> {
    info!("✔ Getting ETH latest block hash from db...");
    get_special_eth_hash_from_db(db, "latest")
}

pub fn get_special_eth_hash_from_db<D: DatabaseInterface>(db: &D, hash_type: &str) -> Result<EthHash> {
    let key = match hash_type {
        "linker" => Ok(EVM_LINKER_HASH_KEY.to_vec()),
        "canon" => Ok(EVM_CANON_BLOCK_HASH_KEY.to_vec()),
        "tail" => Ok(EVM_TAIL_BLOCK_HASH_KEY.to_vec()),
        "anchor" => Ok(EVM_ANCHOR_BLOCK_HASH_KEY.to_vec()),
        "latest" => Ok(EVM_LATEST_BLOCK_HASH_KEY.to_vec()),
        _ => Err(AppError::Custom(format!(
            "✘ Cannot get ETH special hash of type: {}!",
            hash_type
        ))),
    }?;
    trace!("✔ Getting special ETH hash from db of type: {}", hash_type);
    get_eth_hash_from_db(db, &key.to_vec())
}

pub fn get_eth_hash_from_db<D: DatabaseInterface>(db: &D, key: &[Byte]) -> Result<EthHash> {
    trace!("✔ Getting ETH hash from db under key: {}", hex::encode(&key));

    db.get(key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL).map(|mut bytes| {
        bytes.reverse();
        EthHash::from_slice(&bytes)
    })
}

pub fn get_special_eth_block_from_db<D: DatabaseInterface>(db: &D, block_type: &str) -> Result<EthSubmissionMaterial> {
    get_special_eth_hash_from_db(db, block_type).and_then(|block_hash| get_submission_material_from_db(db, &block_hash))
}

pub fn put_eth_hash_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], eth_hash: &EthHash) -> Result<()> {
    db.put(
        key.to_vec(),
        convert_h256_to_bytes(eth_hash),
        MIN_DATA_SENSITIVITY_LEVEL,
    )
}

pub fn eth_block_exists_in_db<D: DatabaseInterface>(db: &D, block_hash: &EthHash) -> bool {
    let key = convert_h256_to_bytes(block_hash);
    info!("✔ Checking for existence of ETH block: {}", hex::encode(&key));
    key_exists_in_db(db, &key, MIN_DATA_SENSITIVITY_LEVEL)
}

pub fn put_eth_submission_material_in_db<D: DatabaseInterface>(
    db: &D,
    eth_submission_material: &EthSubmissionMaterial,
) -> Result<()> {
    let key = convert_h256_to_bytes(&eth_submission_material.get_block_hash()?);
    trace!("✔ Adding block to database under key: {:?}", hex::encode(&key));
    db.put(
        key,
        eth_submission_material.remove_block().to_bytes()?,
        MIN_DATA_SENSITIVITY_LEVEL,
    )
}

pub fn maybe_get_parent_eth_submission_material<D: DatabaseInterface>(
    db: &D,
    block_hash: &EthHash,
) -> Option<EthSubmissionMaterial> {
    debug!("✔ Maybe getting parent ETH block from db...");
    maybe_get_nth_ancestor_eth_submission_material(db, block_hash, 1).ok()?
}

pub fn maybe_get_nth_ancestor_eth_submission_material<D: DatabaseInterface>(
    db: &D,
    block_hash: &EthHash,
    n: u64,
) -> Result<Option<EthSubmissionMaterial>> {
    debug!("✔ Getting {}th ancestor ETH block from db...", n);
    match maybe_get_eth_submission_material_from_db(db, block_hash) {
        None => Ok(None),
        Some(block_and_receipts) => match n {
            0 => Ok(Some(block_and_receipts)),
            _ => maybe_get_nth_ancestor_eth_submission_material(db, &block_and_receipts.get_parent_hash()?, n - 1),
        },
    }
}

pub fn maybe_get_eth_submission_material_from_db<D: DatabaseInterface>(
    db: &D,
    block_hash: &EthHash,
) -> Option<EthSubmissionMaterial> {
    trace!(
        "✔ Maybe getting ETH block and receipts from db under hash: {}",
        block_hash
    );
    match db.get(convert_h256_to_bytes(block_hash), MIN_DATA_SENSITIVITY_LEVEL) {
        Err(_) => None,
        Ok(bytes) => match EthSubmissionMaterial::from_bytes(&bytes) {
            Ok(block_and_receipts) => {
                trace!("✔ Decoded eth block and receipts from db!");
                Some(block_and_receipts)
            },
            Err(_) => {
                error!("✘ Failed to decode eth block and receipts from db!");
                None
            },
        },
    }
}

pub fn get_submission_material_from_db<D: DatabaseInterface>(
    db: &D,
    block_hash: &EthHash,
) -> Result<EthSubmissionMaterial> {
    trace!("✔ Getting ETH block and receipts from db...");
    db.get(convert_h256_to_bytes(block_hash), MIN_DATA_SENSITIVITY_LEVEL)
        .and_then(|bytes| EthSubmissionMaterial::from_bytes(&bytes))
}

pub fn key_exists_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], sensitivity: DataSensitivity) -> bool {
    trace!("✔ Checking for existence of key: {}", hex::encode(key));
    db.get(key.to_vec(), sensitivity).is_ok()
}

pub fn put_eth_gas_price_in_db<D: DatabaseInterface>(db: &D, gas_price: u64) -> Result<()> {
    trace!("✔ Putting ETH gas price of {} in db...", gas_price);
    db.put(
        EVM_GAS_PRICE_KEY.to_vec(),
        gas_price.to_le_bytes().to_vec(),
        MIN_DATA_SENSITIVITY_LEVEL,
    )
}

pub fn get_eth_gas_price_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    trace!("✔ Getting ETH gas price from db...");
    db.get(EVM_GAS_PRICE_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
        .and_then(|bytes| match bytes.len() <= 8 {
            true => {
                let mut array = [0; 8];
                let bytes = &bytes[..array.len()];
                array.copy_from_slice(bytes);
                Ok(u64::from_le_bytes(array))
            },
            false => Err("✘ Too many bytes to convert to u64!".into()),
        })
}

pub fn get_eth_account_nonce_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    trace!("✔ Getting ETH account nonce from db...");
    get_u64_from_db(db, &EVM_ACCOUNT_NONCE_KEY.to_vec())
}

pub fn put_eth_account_nonce_in_db<D: DatabaseInterface>(db: &D, nonce: u64) -> Result<()> {
    trace!("✔ Putting ETH account nonce of {} in db...", nonce);
    put_u64_in_db(db, &EVM_ACCOUNT_NONCE_KEY.to_vec(), nonce)
}

pub fn put_eth_chain_id_in_db<D: DatabaseInterface>(db: &D, chain_id: &EthChainId) -> Result<()> {
    info!("✔ Putting `EthChainId` in db: {}", chain_id);
    db.put(
        EVM_CHAIN_ID_KEY.to_vec(),
        chain_id.to_bytes()?,
        MIN_DATA_SENSITIVITY_LEVEL,
    )
}

pub fn get_eth_chain_id_from_db<D: DatabaseInterface>(db: &D) -> Result<EthChainId> {
    trace!("✔ Getting ETH `chain_id` from db...");
    db.get(EVM_CHAIN_ID_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
        .and_then(|ref bytes| EthChainId::from_bytes(bytes))
}

pub fn put_eth_private_key_in_db<D: DatabaseInterface>(db: &D, pk: &EthPrivateKey) -> Result<()> {
    trace!("✔ Putting ETH private key in db...");
    pk.write_to_database(db, &EVM_PRIVATE_KEY_DB_KEY.to_vec())
}

pub fn get_eth_private_key_from_db<D: DatabaseInterface>(db: &D) -> Result<EthPrivateKey> {
    trace!("✔ Getting ETH private key from db...");
    db.get(EVM_PRIVATE_KEY_DB_KEY.to_vec(), MAX_DATA_SENSITIVITY_LEVEL)
        .and_then(|pk_bytes| {
            let mut array = [0; 32];
            array.copy_from_slice(&pk_bytes[..32]);
            EthPrivateKey::from_slice(&array)
        })
}

pub fn get_erc777_proxy_contract_address_from_db<D: DatabaseInterface>(db: &D) -> Result<EthAddress> {
    trace!("✔ Getting ERC777 proxy contract address from db...");
    match db.get(
        EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY.to_vec(),
        MIN_DATA_SENSITIVITY_LEVEL,
    ) {
        Ok(address_bytes) => Ok(EthAddress::from_slice(&address_bytes[..])),
        Err(_) => {
            debug!("✘ No ERC777 proxy address in db, defaulting to zero ETH address!");
            Ok(EthAddress::zero())
        },
    }
}

#[allow(dead_code)]
pub fn put_erc777_proxy_contract_address_in_db<D: DatabaseInterface>(
    db: &D,
    proxy_contract_address: &EthAddress,
) -> Result<()> {
    trace!("✔ Putting ERC777 proxy contract address in db...");
    put_eth_address_in_db(
        db,
        &EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY.to_vec(),
        proxy_contract_address,
    )
}

pub fn get_public_eth_address_from_db<D: DatabaseInterface>(db: &D) -> Result<EthAddress> {
    trace!("✔ Getting public ETH address from db...");
    db.get(EVM_ADDRESS_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
        .map(|bytes| EthAddress::from_slice(&bytes))
}

pub fn put_public_eth_address_in_db<D: DatabaseInterface>(db: &D, eth_address: &EthAddress) -> Result<()> {
    trace!("✔ Putting public ETH address in db...");
    db.put(
        EVM_ADDRESS_KEY.to_vec(),
        eth_address.as_bytes().to_vec(),
        MIN_DATA_SENSITIVITY_LEVEL,
    )
}

pub fn put_eth_address_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], eth_address: &EthAddress) -> Result<()> {
    db.put(
        key.to_vec(),
        eth_address.as_bytes().to_vec(),
        MIN_DATA_SENSITIVITY_LEVEL,
    )
}

pub fn get_any_sender_nonce_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    trace!("✔ Getting AnySender nonce from db...");
    Ok(
        get_u64_from_db(db, &EVM_ANY_SENDER_NONCE_KEY.to_vec()).unwrap_or_else(|_| {
            info!("✘ Could not find `AnySender` nonce in db, defaulting to `0`");
            0
        }),
    )
}

pub fn put_any_sender_nonce_in_db<D: DatabaseInterface>(db: &D, nonce: u64) -> Result<()> {
    trace!("✔ Putting AnySender nonce of {} in db...", nonce);
    put_u64_in_db(db, &EVM_ANY_SENDER_NONCE_KEY.to_vec(), nonce)
}
