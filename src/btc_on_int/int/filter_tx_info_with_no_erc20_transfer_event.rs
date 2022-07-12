use crate::{
    btc_on_int::int::btc_tx_info::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
    chains::eth::{
        eth_contracts::erc20_token::{Erc20TokenTransferEvent, Erc20TokenTransferEvents, ToErc20TokenTransferEvent},
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

impl ToErc20TokenTransferEvent for BtcOnIntBtcTxInfo {
    fn to_erc20_token_transfer_event(&self) -> Erc20TokenTransferEvent {
        Erc20TokenTransferEvent::new(self.amount_in_wei, self.to, self.from, self.token_address)
    }
}

pub fn maybe_filter_those_with_no_corresponding_erc20_transfer_event<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out `BtcOnIntBtcTxInfos` which don't have corresponding ERC20 transfer events ...");
    state
        .eth_db_utils
        .get_eth_canon_block_from_db()
        .and_then(|canon_block| Erc20TokenTransferEvents::from_eth_submission_material(&canon_block))
        .map(|erc20_transfers| erc20_transfers.filter_if_no_transfer_event(&state.btc_on_int_btc_tx_infos))
        .map(BtcOnIntBtcTxInfos::new)
        .and_then(|filtered_tx_infos| state.replace_btc_on_int_btc_tx_infos(filtered_tx_infos))
}
