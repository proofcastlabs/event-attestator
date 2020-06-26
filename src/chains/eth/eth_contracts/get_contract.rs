use ethabi::Contract as EthContract;
use crate::{
    types::Result,
    errors::AppError,
};

pub fn instantiate_contract_from_abi(abi: &str) -> Result<EthContract> {
    match EthContract::load(abi.as_bytes()) {
        Ok(contract) => Ok(contract),
        Err(e) => Err(AppError::Custom(format!("✘ Error instantiating contract from ABI fragment: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::change_erc777_pnetwork_address::CHANGE_PNETWORK_ABI;

    #[test]
    fn should_instantiate_pnetwork_contract_from_abi() {
        if let Err(e) = instantiate_contract_from_abi(CHANGE_PNETWORK_ABI) {
            panic!("Error instantiating contract from abi: {}", e);
        }
    }
}
