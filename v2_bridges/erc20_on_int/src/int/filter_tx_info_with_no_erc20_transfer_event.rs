use crate::int::eth_tx_info::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos};

impl_to_erc20_token_event!(
    Erc20OnIntEthTxInfo,
    host_token_amount,
    token_recipient,
    token_sender,
    evm_token_address
);

make_erc20_token_event_filterer_v2!(EthState<D>, evm_db_utils, Erc20OnIntEthTxInfos);
