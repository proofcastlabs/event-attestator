use crate::types::Result;
use ethabi::Contract as EthContract;

pub fn instantiate_contract_from_abi(abi: &str) -> Result<EthContract> {
    Ok(EthContract::load(abi.as_bytes())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::eth_contracts::erc777_proxy::ERC777_PROXY_ABI;

    #[test]
    fn should_instantiate_pnetwork_contract_from_abi() {
        if let Err(e) = instantiate_contract_from_abi(ERC777_PROXY_ABI) {
            panic!("Error instantiating contract from abi: {}", e);
        }
    }
}
