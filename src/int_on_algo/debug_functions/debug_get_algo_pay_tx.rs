use std::str::FromStr;

use rust_algorand::{AlgorandAddress, AlgorandGenesisId, AlgorandTransaction, MicroAlgos};

use crate::{
    chains::{algo::algo_database_utils::AlgoDbUtils, eth::eth_database_utils::EthDbUtils},
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    int_on_algo::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
    utils::strip_hex_prefix,
};

/// # Debug Get Algo Pay Tx
///
/// This function will create an Algorand `pay` tx type using the passed in arguments and signed
/// by the algorand key saved in the encrypted database.
///
/// __NOTE:__ This function will _not_ increment the ALGO signature nonce!
pub fn debug_get_algo_pay_tx<D: DatabaseInterface>(
    db: &D,
    first_valid: u64,
    genesis_id: &str,
    fee: u64,
    receiver: &str,
    note: &str,
    amount: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Getting ALGO pay tx...");
    let algo_db_utils = AlgoDbUtils::new(db);
    // TODO If the note is valid hex, use it raw, else if is valid utf8, convert it to bytes.
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &algo_db_utils))
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::IntOnAlgo, signature, debug_command_hash))
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
