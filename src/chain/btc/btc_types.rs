#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfoJson {
    pub nonce: u64,
    pub address: String,
    pub btc_deposit_address: String,
    pub address_and_nonce_hash: String,
}

impl DepositAddressInfoJson {
    pub fn new(
        nonce: u64,
        address: String,
        btc_deposit_address: String,
        address_and_nonce_hash: String,
    ) -> Self {
        DepositAddressInfoJson {
            nonce,
            address,
            btc_deposit_address,
            address_and_nonce_hash,
        }
    }
}
