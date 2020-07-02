use ethabi::Token;
use ethereum_types::Address as EthAddress;
use crate::{
    types::{
        Bytes,
        Result,
    },
    chains::eth::eth_contracts::get_contract::instantiate_contract_from_abi,
};

pub const ERC777_CHANGE_PNETWORK_GAS_LIMIT: usize = 30_000;
const ERC777_CHANGE_PNETWORK_FXN_NAME: &str = "changePNetwork";

pub const ERC777_ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"newPNetwork\",\"type\":\"address\"}],\"name\":\"changePNetwork\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"signature\":\"0xfd4add66\"}]";

pub fn encode_erc777_change_pnetwork_fxn_data(new_ptoken_address: EthAddress) -> Result<Bytes> {
    Ok(
        instantiate_contract_from_abi(ERC777_ABI)?
            .function(ERC777_CHANGE_PNETWORK_FXN_NAME)?
            .encode_input(&[Token::Address(new_ptoken_address)])?
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_erc777_change_pnetwork_fxn_data() {
        let expected_result = "fd4add66000000000000000000000000736661736533bcfc9cc35649e6324acefb7d32c1";
        let address = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());
        let result = encode_erc777_change_pnetwork_fxn_data(address).unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }
}
