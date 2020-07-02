use ethabi::{encode, Token};
use ethereum_types::{
    U256,
    Address as EthAddress,
};
use crate::{
    types::{
        Bytes,
        Result,
    },
    btc_on_eth::eth::eth_crypto::eth_private_key::EthPrivateKey,
    chains::eth::eth_contracts::get_contract::instantiate_contract_from_abi,
};

const MINT_BY_PROXY_FXN_NAME: &str = "mintByProxy";

pub const ERC777_PROXY_ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"_recipient\",\"type\":\"address\"},{\"name\":\"_amount\",\"type\":\"uint256\"},{\"name\":\"_nonce\",\"type\":\"uint256\"},{\"name\":\"_signature\",\"type\":\"bytes\"}],\"name\":\"mintByProxy\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"signature\":\"0x7ad6ae47\"}]";

pub fn encode_mint_by_proxy_tx_data(
    eth_private_key: &EthPrivateKey,
    token_recipient: EthAddress,
    token_amount: U256,
    any_sender_nonce: u64,
) -> Result<Bytes> {
    let proxy_signature = eth_private_key
        .sign_eth_prefixed_msg_bytes(encode(&[
            Token::Address(EthAddress::from_slice(token_recipient.as_bytes())),
            Token::Uint(token_amount),
            Token::Uint(any_sender_nonce.into()),
        ]))?.to_vec();
    let proxy_tokens = [
        Token::Address(EthAddress::from_slice(token_recipient.as_bytes())),
        Token::Uint(token_amount),
        Token::Uint(any_sender_nonce.into()),
        Token::Bytes(proxy_signature),
    ];
    Ok(instantiate_contract_from_abi(ERC777_PROXY_ABI)?.function(MINT_BY_PROXY_FXN_NAME)?.encode_input(&proxy_tokens)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_mint_by_proxy_tx_data() {
        let eth_private_key = EthPrivateKey::from_slice([
            132, 23, 52, 203, 67, 154, 240, 53, 117, 195, 124, 41, 179, 50, 97, 159, 61,
            169, 234, 47, 186, 237, 88, 161, 200, 177, 24, 142, 207, 242, 168, 221,
        ]).unwrap();
        let token_recipient = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());
        let any_sender_nonce = 0;
        let token_amount = U256::from(1337);
        let result = encode_mint_by_proxy_tx_data(&eth_private_key, token_recipient, token_amount, any_sender_nonce)
            .unwrap();
        let expected_result = "7ad6ae47000000000000000000000000736661736533bcfc9cc35649e6324acefb7d32c10000000000000000000000000000000000000000000000000000000000000539000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000041fa1150574c2f9bbc0addffee7be31317370fc941f3853476c9830f117b4f31f51c77c0b270e7141033882d5701822586318c5b455f674ab93c5066fc280802991b00000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(hex::encode(result), expected_result);
    }
}
