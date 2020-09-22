use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::eth_database_utils::{
        put_eth_canon_block_in_db,
        get_eth_canon_block_from_db,
    },
};

pub fn remove_receipts_from_canon_block_and_save_in_db<D>(db: &D) -> Result<()> where D: DatabaseInterface {
    get_eth_canon_block_from_db(db).and_then(|block| put_eth_canon_block_in_db(db, &block.remove_receipts()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_utils::get_test_database,
        btc_on_eth::eth::eth_test_utils::get_sample_eth_submission_material,
    };

    #[test]
    fn should_remove_receipts_from_canon_block() {
        let db = get_test_database();
        let canon_block = get_sample_eth_submission_material();
        put_eth_canon_block_in_db(&db, &canon_block)
            .unwrap();
        let num_receipts_before = get_eth_canon_block_from_db(&db)
            .unwrap()
            .receipts
            .len();
        if let Err(e) = remove_receipts_from_canon_block_and_save_in_db(&db) {
            panic!("Error maybe removing receipts from canon: {}", e);
        }
        let num_receipts_after = get_eth_canon_block_from_db(&db)
            .unwrap()
            .receipts
            .len();
        assert!(num_receipts_before > 0);
        assert_eq!(num_receipts_after, 0);
    }

    #[test]
    fn should_not_err_if_canon_has_no_receipts() {
        let db = get_test_database();
        let canon_block = get_sample_eth_submission_material().remove_receipts();
        put_eth_canon_block_in_db(&db, &canon_block)
            .unwrap();
        let num_receipts_before = get_eth_canon_block_from_db(&db)
            .unwrap()
            .receipts
            .len();
        if let Err(e) = remove_receipts_from_canon_block_and_save_in_db(&db) {
            panic!("Error maybe removing receipts from canon: {}", e);
        }
        let num_receipts_after = get_eth_canon_block_from_db(&db)
            .unwrap()
            .receipts
            .len();
        assert_eq!(num_receipts_before, 0);
        assert_eq!(num_receipts_after, 0);
    }
}
