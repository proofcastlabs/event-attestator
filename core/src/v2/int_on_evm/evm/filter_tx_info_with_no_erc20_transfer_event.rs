use crate::int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos};

impl_to_erc20_token_event!(
    IntOnEvmIntTxInfo,
    host_token_amount,
    vault_address,
    token_sender,
    evm_token_address
);

make_erc20_token_event_filterer!(EthState<D>, evm_db_utils, IntOnEvmIntTxInfos);
