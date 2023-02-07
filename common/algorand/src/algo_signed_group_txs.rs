
use derive_more::{Constructor, Deref};
use rust_algorand::AlgorandTxGroup;

#[derive(Clone, Debug, Eq, PartialEq, Constructor, Deref)]
#[derive(Default)]
pub struct AlgoSignedGroupTxs(Vec<(String, AlgorandTxGroup)>);



#[derive(Clone, Debug, Eq, PartialEq, Constructor)]
pub struct AlgoSignedGroupTx((String, AlgorandTxGroup));
