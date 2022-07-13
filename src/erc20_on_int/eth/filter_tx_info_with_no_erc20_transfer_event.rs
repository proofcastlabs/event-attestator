use crate::erc20_on_int::eth::int_tx_info::{Erc20OnIntIntTxInfo, Erc20OnIntIntTxInfos};

impl_to_erc20_token_event!(
    Erc20OnIntIntTxInfo,
    native_token_amount,
    vault_address,
    token_sender,
    eth_token_address
);

make_erc20_token_event_filterer!(EthState<D>, eth_db_utils, Erc20OnIntIntTxInfos);
