use crate::int::eth_tx_info::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos};

impl_to_relevant_events!(
    Erc20OnIntEthTxInfo,
    host_token_amount,
    token_recipient,
    token_sender,
    evm_token_address
);

make_erc20_token_event_filterer!(EthState<D>, evm_db_utils, Erc20OnIntEthTxInfos);
