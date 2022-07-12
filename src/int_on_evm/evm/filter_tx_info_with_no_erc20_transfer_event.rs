use crate::{
    chains::eth::{
        eth_contracts::erc20_token::{Erc20TokenTransferEvent, Erc20TokenTransferEvents, ToErc20TokenTransferEvent},
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
    },
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl ToErc20TokenTransferEvent for IntOnEvmIntTxInfo {
    fn to_erc20_token_transfer_event(&self) -> Erc20TokenTransferEvent {
        Erc20TokenTransferEvent::new(
            self.native_token_amount,
            self.vault_address,
            self.token_sender,
            self.evm_token_address,
        )
    }
}

pub fn filter_tx_info_with_no_erc20_transfer_event<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Filtering out `IntOnEvmIntTxInfo`s which don't have corresponding ERC20 transfer events ...");
    state
        .evm_db_utils
        .get_eth_canon_block_from_db()
        .map(|canon_block_submission_material| {
            Erc20TokenTransferEvents::filter_if_no_transfer_event_in_submission_material(
                &canon_block_submission_material,
                &state.int_on_evm_int_tx_infos,
            )
        })
        .map(IntOnEvmIntTxInfos::new)
        .and_then(|filtered_tx_infos| state.replace_int_on_evm_int_tx_infos(filtered_tx_infos))
}
