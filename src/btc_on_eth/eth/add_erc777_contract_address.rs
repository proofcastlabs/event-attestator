use crate::{
    btc_on_eth::check_core_is_initialized::check_core_is_initialized,
    chains::{
        btc::btc_database_utils::BtcDbUtils,
        eth::{
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_utils::convert_hex_to_eth_address,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Maybe Add ERC777 Contract Address
///
/// This function will add a passed in ETH address to the encrypted database since the ETH
/// initialization no longer creates one. Until this step has been carried out after ETH core
/// initialization, the `get_enclave_state` command will error with a message telling you to call
/// this function.
///
/// ### BEWARE:
/// The ERC777 contract can only be set ONCE. Further attempts to do so will not succeed.
pub fn maybe_add_erc777_contract_address<D: DatabaseInterface>(db: &D, hex_address: &str) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);
    let btc_db_utils = BtcDbUtils::new(db);
    check_core_is_initialized(&eth_db_utils, &btc_db_utils)
        .and_then(|_| db.start_transaction())
        .and_then(|_| convert_hex_to_eth_address(hex_address))
        .and_then(|ref address| eth_db_utils.put_btc_on_eth_smart_contract_address_in_db(address))
        .and_then(|_| db.end_transaction())
        .map(|_| "{add_erc777_address_success:true}".to_string())
}
