use std::str::FromStr;

use common::{
    chains::algo::algo_database_utils::AlgoDbUtils,
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    traits::DatabaseInterface,
    types::Result,
    utils::strip_hex_prefix,
};
use function_name::named;
use rust_algorand::{AlgorandAddress, AlgorandGenesisId, AlgorandTransaction, MicroAlgos};

use crate::constants::CORE_TYPE;

/// # Debug Get Algo Pay Tx
///
/// This function will create an Algorand `pay` tx type using the passed in arguments and signed
/// by the algorand key saved in the encrypted database.
///
/// __NOTE:__ This function will _not_ increment the ALGO signature nonce!
#[allow(clippy::too_many_arguments)]
#[named]
pub fn debug_get_algo_pay_tx<D: DatabaseInterface>(
    db: &D,
    first_valid: u64,
    genesis_id: &str,
    fee: u64,
    receiver: &str,
    note: &str,
    amount: u64,
    signature: &str,
) -> Result<String> {
    info!("✔ Getting ALGO pay tx...");
    let algo_db_utils = AlgoDbUtils::new(db);
    // TODO If the note is valid hex, use it raw, else if is valid utf8, convert it to bytes.
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| {
            get_debug_command_hash!(
                function_name!(),
                &first_valid,
                genesis_id,
                &fee,
                receiver,
                note,
                &amount
            )()
        })
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| {
            let pk = algo_db_utils.get_algo_private_key()?;
            let note_bytes = hex::decode(strip_hex_prefix(note))?;
            let tx = AlgorandTransaction::new_payment_tx(
                amount,
                MicroAlgos::new(fee),
                if note_bytes.is_empty() { None } else { Some(note_bytes) },
                first_valid,
                pk.to_address()?,
                AlgorandAddress::from_str(receiver)?,
                AlgorandGenesisId::from_str(genesis_id)?.hash()?,
                None,
            )?
            .sign(&pk)?
            .to_hex()?;
            db.end_transaction()?;
            Ok(tx)
        })
}
