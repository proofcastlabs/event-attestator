use crate::int_on_eos::int::eos_tx_info::{IntOnEosEosTxInfo, IntOnEosEosTxInfos};

impl_to_erc20_token_event!(
    IntOnEosEosTxInfo,
    token_amount,
    vault_address,
    token_sender,
    eth_token_address
);

make_erc20_token_event_filterer!(EthState<D>, eth_db_utils, IntOnEosEosTxInfos);
