use crate::{
    chains::eth::{
        eth_constants::{ETH_LINKER_HASH_KEY, PTOKEN_GENESIS_HASH_KEY},
        eth_database_utils::EthDbUtilsExt,
        eth_types::EthHash,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_linker_hash_or_genesis_hash<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<EthHash> {
    match db_utils.get_hash_from_db_via_hash_key(EthHash::from_slice(&ETH_LINKER_HASH_KEY[..]))? {
        Some(hash) => Ok(hash),
        None => {
            info!("✔ No linker-hash set yet, using pToken genesis hash...");
            Ok(EthHash::from_slice(&PTOKEN_GENESIS_HASH_KEY[..]))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::eth::eth_database_utils::EthDbUtils, test_utils::get_test_database};

    #[test]
    fn get_linker_or_genesis_should_get_linker_hash_from_db_if_extant() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let linker_hash = EthHash::random();
        eth_db_utils.put_eth_linker_hash_in_db(linker_hash).unwrap();
        let result = get_linker_hash_or_genesis_hash(&eth_db_utils).unwrap();
        assert_eq!(result, linker_hash);
    }

    #[test]
    fn get_linker_or_genesis_should_get_genesis_hash_if_linker_not_set() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let result = get_linker_hash_or_genesis_hash(&eth_db_utils).unwrap();
        assert_eq!(result, EthHash::from_slice(&PTOKEN_GENESIS_HASH_KEY[..]));
    }
}
