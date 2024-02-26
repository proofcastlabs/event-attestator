use common::{
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;

use crate::{
    core_initialization::{generate_and_put_incremerkle_in_db, put_eos_latest_block_info_in_db, EosInitJson},
    eos_database_utils::EosDbUtils,
};

/// # Debug Update Incremerkle
///
/// This function will take an EOS initialization JSON as its input and use it to create an
/// incremerkle valid for the block number in the JSON. It will then REPLACE the incremerkles
/// in the encrypted database with this one.
///
/// ### BEWARE:
/// Changing the incremerkle changes the last block the enclave has seen and so can easily lead to
/// transaction replays. Use with extreme caution and only if you know exactly what you are doing
/// and why.
#[named]
pub fn debug_update_incremerkle<D: DatabaseInterface>(
    db: &D,
    eos_init_json_str: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("debug updating blockroot merkle...");
    let init_json = EosInitJson::from_json_string(eos_init_json_str)?;
    let eos_db_utils = EosDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &init_json, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
        .and_then(|_| put_eos_latest_block_info_in_db(&eos_db_utils, &init_json.block))
        .and_then(|_| generate_and_put_incremerkle_in_db(&eos_db_utils, &init_json))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_update_blockroot_merkle_success:true}".to_string()))
        .map(prepend_debug_output_marker_to_string)
}
