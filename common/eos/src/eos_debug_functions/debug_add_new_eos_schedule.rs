use common::{
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;

use crate::{eos_database_utils::EosDbUtils, eos_producer_schedule::EosProducerScheduleV2};

/// # Debug Add New Eos Schedule
///
/// Adds a new EOS schedule to the core's encrypted database.
#[named]
pub fn debug_add_new_eos_schedule<D: DatabaseInterface>(
    db: &D,
    schedule_json: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug adding new EOS schedule...");
    let schedule = EosProducerScheduleV2::from_json(schedule_json)?;
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &schedule, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
        .and_then(|_| EosDbUtils::new(db).put_eos_schedule_in_db(&schedule))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_adding_eos_schedule_success:true}".to_string()))
        .map(prepend_debug_output_marker_to_string)
}
