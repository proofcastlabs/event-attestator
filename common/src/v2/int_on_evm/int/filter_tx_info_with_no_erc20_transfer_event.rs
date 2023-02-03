use crate::int_on_evm::int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos};

impl_to_erc20_token_event!(
    IntOnEvmEvmTxInfo,
    native_token_amount,
    vault_address,
    token_sender,
    eth_token_address
);

make_erc20_token_event_filterer_v2!(EthState<D>, eth_db_utils, IntOnEvmEvmTxInfos);
