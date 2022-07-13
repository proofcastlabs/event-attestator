use crate::{
    btc_on_int::int::btc_tx_info::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
    chains::eth::{
        eth_contracts::erc20_token::Erc20TokenTransferEvents,
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

impl_to_erc20_token_event!(BtcOnIntBtcTxInfo, amount_in_wei, to, from, token_address);

pub fn filter_tx_info_with_no_erc20_transfer_event<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Filtering out `BtcOnIntBtcTxInfos` which don't have corresponding ERC20 transfer events ...");
    state
        .eth_db_utils
        .get_eth_canon_block_from_db()
        .map(|canon_block_submission_material| {
            Erc20TokenTransferEvents::filter_if_no_transfer_event_in_submission_material(
                &canon_block_submission_material,
                &state.btc_on_int_btc_tx_infos,
            )
        })
        .map(BtcOnIntBtcTxInfos::new)
        .and_then(|filtered_tx_infos| state.replace_btc_on_int_btc_tx_infos(filtered_tx_infos))
}
