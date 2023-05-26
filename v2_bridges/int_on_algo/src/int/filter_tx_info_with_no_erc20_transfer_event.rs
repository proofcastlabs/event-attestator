use crate::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos};

impl_to_erc20_token_event!(
    IntOnAlgoAlgoTxInfo,
    native_token_amount,
    vault_address,
    token_sender,
    int_token_address
);

make_erc20_token_event_filterer!(EthState<D>, eth_db_utils, IntOnAlgoAlgoTxInfos);
