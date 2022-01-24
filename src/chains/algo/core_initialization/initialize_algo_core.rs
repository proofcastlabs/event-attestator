use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn initialize_algo_core<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    /* TODO
     *
    So think. do we pass in the asset ID etc? Generate it outside then pass executive control to the core?

        parse material
        chain id
        rm ALL receipts from block
        put block in db
        save canon to tip length
        save anchor block hash
        save latest block hash
        save canon block hash
        generate & save private key
        generate & save address
        tail block hash
        gas price/fee
        account nonce

    */
    Ok(state)
}
