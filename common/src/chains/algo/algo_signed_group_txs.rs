use derive_more::{Constructor, Deref};
use rust_algorand::AlgorandTxGroup;

#[derive(Clone, Debug, Eq, PartialEq, Constructor, Deref)]
pub struct AlgoSignedGroupTxs(Vec<AlgoSignedGroupTx>);

#[derive(Clone, Debug, Eq, PartialEq, Constructor)]
pub struct AlgoSignedGroupTx {
    pub signed_tx: String,
    pub group_tx: AlgorandTxGroup,
}
