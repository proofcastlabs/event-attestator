use crate::{
    chains::eos::core_initialization::eos_init_utils::{
        generate_and_put_incremerkle_in_db,
        put_eos_latest_block_info_in_db,
        EosInitJson,
    },
    check_debug_mode::check_debug_mode,
    traits::DatabaseInterface,
    types::Result,
};

pub fn update_incremerkle<D: DatabaseInterface>(db: &D, init_json: &EosInitJson) -> Result<String> {
    info!("✔ Debug updating blockroot merkle...");
    check_debug_mode()
        .and_then(|_| put_eos_latest_block_info_in_db(db, &init_json.block))
        .and_then(|_| db.start_transaction())
        .and_then(|_| generate_and_put_incremerkle_in_db(db, &init_json.blockroot_merkle))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_update_blockroot_merkle_success:true}".to_string()))
}
