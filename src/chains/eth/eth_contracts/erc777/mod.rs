use ethabi::Token;
use ethereum_types::{
    U256,
    Address as EthAddress,
};
use crate::{
    types::{
        Bytes,
        Result,
    },
    chains::eth::eth_contracts::encode_fxn_call,
};

pub const ERC777_CHANGE_PNETWORK_GAS_LIMIT: usize = 30_000;
pub const EMPTY_DATA: Bytes = vec![];

pub const ERC777_CHANGE_PNETWORK_ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"newPNetwork\",\"type\":\"address\"}],\"name\":\"changePNetwork\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"signature\":\"0xfd4add66\"}]";

pub const ERC777_MINT_WITH_NO_DATA_ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

pub fn encode_erc777_change_pnetwork_fxn_data(new_ptoken_address: EthAddress) -> Result<Bytes> { // TODO Take a reference!
    encode_fxn_call(ERC777_CHANGE_PNETWORK_ABI, "changePNetwork", &[Token::Address(new_ptoken_address)])
}

pub fn encode_erc777_mint_with_no_data_fxn(
    recipient: &EthAddress,
    value: &U256,
) -> Result<Bytes> {
    encode_fxn_call(ERC777_MINT_WITH_NO_DATA_ABI, "mint", &[Token::Address(*recipient), Token::Uint(*value)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        get_sample_eth_address,
    };

    #[test]
    fn should_encode_erc777_change_pnetwork_fxn_data() {
        let expected_result = "fd4add66000000000000000000000000736661736533bcfc9cc35649e6324acefb7d32c1";
        let address = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());
        let result = encode_erc777_change_pnetwork_fxn_data(address).unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_encode_erc777_mint_with_no_data_fxn () {
        let expected_result = "40c10f190000000000000000000000001739624f5cd969885a224da84418d12b8570d61a0000000000000000000000000000000000000000000000000000000000000001".to_string();
        let recipient = EthAddress::from_slice(&hex::decode("1739624f5cd969885a224da84418d12b8570d61a").unwrap());
        let amount = U256::from_dec_str("1").unwrap();
        let result = encode_erc777_mint_with_no_data_fxn(&recipient, &amount).unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }
}
