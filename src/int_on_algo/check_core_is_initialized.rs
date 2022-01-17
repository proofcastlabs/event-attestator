use crate::{
    chains::{
        algo::algo_database_utils::AlgoDbUtils,
        eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn check_core_is_initialized<D: DatabaseInterface>(
    int_db_utils: &EthDbUtils<D>,
    algo_db_utils: &AlgoDbUtils<D>,
) -> Result<()> {
    Ok(()) // FIXME
}
