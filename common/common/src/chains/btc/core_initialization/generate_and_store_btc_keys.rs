use crate::{
    chains::btc::{
        btc_crypto::btc_private_key::BtcPrivateKey,
        btc_database_utils::BtcDbUtils,
        core_initialization::btc_init_utils::get_btc_network_from_arg,
        BtcState,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn generate_and_store_btc_keys<D: DatabaseInterface>(network: &str, db_utils: &BtcDbUtils<D>) -> Result<()> {
    let pk = BtcPrivateKey::generate_random(get_btc_network_from_arg(network))?;
    db_utils
        .put_btc_private_key_in_db(&pk)
        .and_then(|_| {
            db_utils.put_btc_pub_key_slice_in_db(&db_utils.get_btc_private_key_from_db()?.to_public_key_slice())
        })
        .and_then(|_| db_utils.put_btc_address_in_db(&db_utils.get_btc_private_key_from_db()?.to_p2pkh_btc_address()))
}

pub fn generate_and_store_btc_keys_and_return_state<'a, D: DatabaseInterface>(
    network: &str,
    state: BtcState<'a, D>,
) -> Result<BtcState<'a, D>> {
    info!("✔ Generating & storing BTC private key...");
    generate_and_store_btc_keys(network, &state.btc_db_utils).and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::btc::btc_database_utils::BtcDbUtils, test_utils::get_test_database};

    // NOTE: This was the original way the BTC keys were stored. Note the ignored test below to
    // show how it fails. Note also the hack above in the real function and how its test passes.
    fn generate_and_store_btc_keys_broken<D: DatabaseInterface>(network: &str, db_utils: &BtcDbUtils<D>) -> Result<()> {
        let pk = BtcPrivateKey::generate_random(get_btc_network_from_arg(network))?;
        db_utils
            .put_btc_private_key_in_db(&pk)
            .and_then(|_| db_utils.put_btc_pub_key_slice_in_db(&pk.to_public_key_slice()))
            .and_then(|_| db_utils.put_btc_address_in_db(&pk.to_p2pkh_btc_address()))
    }

    #[ignore]
    #[test]
    fn should_show_btc_private_key_db_save_bug() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let network_str = "Bitcoin";
        let network = get_btc_network_from_arg(network_str);
        db_utils.put_btc_network_in_db(network).unwrap();
        generate_and_store_btc_keys_broken(network_str, &db_utils).unwrap();
        let pk_from_db = db_utils.get_btc_private_key_from_db().unwrap();
        let address_from_db = db_utils.get_btc_address_from_db().unwrap();
        let address_from_pk_from_db = pk_from_db.to_p2pkh_btc_address();
        assert_eq!(address_from_db, address_from_pk_from_db); // FIXME: This should not fail!
    }

    #[test]
    fn should_generate_and_store_btc_keys() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let network_str = "Bitcoin";
        let network = get_btc_network_from_arg(network_str);
        db_utils.put_btc_network_in_db(network).unwrap();
        generate_and_store_btc_keys(network_str, &db_utils).unwrap();
        let pk_from_db = db_utils.get_btc_private_key_from_db().unwrap();
        let address_from_db = db_utils.get_btc_address_from_db().unwrap();
        let address_from_pk_from_db = pk_from_db.to_p2pkh_btc_address();
        assert_eq!(address_from_db, address_from_pk_from_db);
    }
}
