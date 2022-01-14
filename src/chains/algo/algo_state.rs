use crate::{
    chains::{algo::algo_database_utils::AlgoDbUtils, eth::eth_database_utils::EthDbUtils},
    traits::DatabaseInterface,
};

#[derive(Clone, PartialEq, Eq)]
pub struct AlgoState<'a, D: DatabaseInterface> {
    db: &'a D,
    pub eth_db_utils: EthDbUtils<'a, D>,
    pub algo_db_utils: AlgoDbUtils<'a, D>,
}

impl<'a, D: DatabaseInterface> AlgoState<'a, D> {
    pub fn init(db: &'a D) -> Self {
        Self {
            db,
            eth_db_utils: EthDbUtils::new(db),
            algo_db_utils: AlgoDbUtils::new(db),
        }
    }
}
