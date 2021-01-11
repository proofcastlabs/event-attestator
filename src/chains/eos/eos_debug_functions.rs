use crate::{
    chains::eos::{
        core_initialization::eos_init_utils::{
            generate_and_put_incremerkle_in_db,
            put_eos_latest_block_info_in_db,
            EosInitJson,
        },
        eos_database_utils::put_eos_schedule_in_db,
        parse_eos_schedule::parse_v2_schedule_string_to_v2_schedule,
    },
    check_debug_mode::check_debug_mode,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

pub fn update_incremerkle<D: DatabaseInterface>(db: &D, init_json: &EosInitJson) -> Result<String> {
    info!("✔ Debug updating blockroot merkle...");
    check_debug_mode()
        .and_then(|_| put_eos_latest_block_info_in_db(db, &init_json.block))
        .and_then(|_| db.start_transaction())
        .and_then(|_| generate_and_put_incremerkle_in_db(db, &init_json.blockroot_merkle))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_update_blockroot_merkle_success:true}".to_string()))
        .map(prepend_debug_output_marker_to_string)
}

pub fn add_new_eos_schedule<D: DatabaseInterface>(db: D, schedule_json: &str) -> Result<String> {
    info!("✔ Debug adding new EOS schedule...");
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| parse_v2_schedule_string_to_v2_schedule(&schedule_json))
        .and_then(|schedule| put_eos_schedule_in_db(&db, &schedule))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_adding_eos_schedule_succeeded:true}".to_string()))
        .map(prepend_debug_output_marker_to_string)
}
