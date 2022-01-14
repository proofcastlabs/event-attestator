use crate::traits::DatabaseInterface;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgoDbUtils<'a, D: DatabaseInterface> {
    db: &'a D,
}

impl<'a, D: DatabaseInterface> AlgoDbUtils<'a, D> {
    pub fn new(db: &'a D) -> Self {
        Self { db }
    }
}
