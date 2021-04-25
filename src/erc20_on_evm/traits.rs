use ethereum_types::U256;

pub trait FeeCalculator {
    fn get_amount(&self) -> U256;

    fn calculate_fee(&self, fee_basis_points: u64) -> U256 {
        (self.get_amount() * U256::from(fee_basis_points)) / U256::from(10_000)
    }
}

pub trait FeesCalculator {
    fn get_fees(&self, fee_basis_points: u64) -> Vec<U256>;

    fn calculate_fees(&self, fee_basis_points: u64) -> (Vec<U256>, U256) {
        let fees = self.get_fees(fee_basis_points);
        let total_fee = fees.iter().fold(U256::zero(), |a, b| a + b);
        info!("✔      Fees: {:?}", fees);
        info!("✔ Total fee: {:?}", fees);
        (fees, total_fee)
    }
}
