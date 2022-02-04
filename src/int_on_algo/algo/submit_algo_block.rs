use crate::{
    chains::algo::{algo_state::AlgoState, algo_submission_material::parse_algo_submission_material_and_put_in_state},
    traits::DatabaseInterface,
    types::Result,
};
<<<<<<< HEAD
=======
// So the setup will be thus:
// Anyone can create an asset, with this enclave address as the:
// Manager account - the only account how can change/reconfigure the asset
// Reserve address - where the created tokens go to (instead of the creator account). Transfers out of here are "mints",
// and transfers back to here are redeems. Freeze address - either this enclave or empty string. This account can then
// freeze people. Clawback address - either this enclave or empty string.
//
// Setting the reserve address to this enclave can ONLY be done if this enclave has signed a tx
// saying it's happy to accept the asset. Nice.
//
// So then the dictionary will be a list of INT vault tokens mapped to asset IDs. Now the enclave
// can search for transactions where it is the recipient of some asset, which will count as a
// redeem and proceed from there.
//
// The other side of the enclave can then use this address to sign txs to send the asset from this
// reserver account, which counts as a mint.
//
// Nice!
>>>>>>> 75c20a6a (ref(eth): <- make diversion fxn macro neater)

/// Submit Algo Block To Core
///
/// The main submission pipeline. Submitting an Algorand block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ALGO
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain pertinent transactions to the redeem addres  the enclave is watching, an INT
/// transaction will be signed & returned to the caller.
pub fn submit_algo_block_to_core<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting ALGO block to core...");
    parse_algo_submission_material_and_put_in_state(block_json_string, AlgoState::init(&db))
        .map(|_| "done!".to_string())
}
