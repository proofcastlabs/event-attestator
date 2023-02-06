use crate::int::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos};

impl_to_erc20_token_event!(BtcOnIntBtcTxInfo, amount_in_wei, to, from, token_address);

make_erc20_token_event_filterer_v2!(EthState<D>, eth_db_utils, BtcOnIntBtcTxInfos);
