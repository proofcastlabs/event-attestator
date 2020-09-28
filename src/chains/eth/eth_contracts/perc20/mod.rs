use ethabi::Token;
use ethereum_types::{
    U256,
    Address as EthAddress,
};
use crate::{
    chains::eth::eth_contracts::encode_fxn_call,
    types::{
        Bytes,
        Result,
    },
};

pub const EMPTY_DATA: Bytes = vec![];
pub const PERC20_PEGOUT_GAS_LIMIT: usize = 100_000; // FIXME: Finesse this once we know the value required!

pub const PERC20_PEGOUT_ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_tokenRecipient\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"_tokenAddress\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"_tokenAmount\",\"type\":\"uint256\"}],\"name\":\"pegOut\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

pub fn encode_perc20_peg_out_fxn_data(
    recipient: EthAddress,
    token_contract_address: EthAddress,
    amount: U256,
) -> Result<Bytes> {
    encode_fxn_call(
        PERC20_PEGOUT_ABI,
        "pegOut",
        &[Token::Address(recipient), Token::Address(token_contract_address), Token::Uint(amount)]
    )
}
