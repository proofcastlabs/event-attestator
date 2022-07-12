use crate::{
    btc_on_int::int::btc_tx_info::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
    chains::eth::{
        eth_contracts::erc20_token::{Erc20TokenTransferEvent, Erc20TokenTransferEvents},
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    traits::DatabaseInterface,
    types::Result,
};

impl BtcOnIntBtcTxInfo {
    pub fn to_erc20_transfer_event(&self) -> Erc20TokenTransferEvent {
        Erc20TokenTransferEvent::new(self.amount_in_wei, self.to, self.from, self.token_address)
    }
}

impl BtcOnIntBtcTxInfos {
    // FIXME Need to make this smarter so same peg out info > 1 in a block requires BOTH transfer
    // events!
    pub fn filter_those_with_no_corresponding_erc20_transfer_event(
        &self,
        submission_material: &EthSubmissionMaterial,
    ) -> Result<Self> {
        info!("✔ Number of `BtcOnIntBtcTxInfos` before: {}", self.len());
        Erc20TokenTransferEvents::from_eth_submission_material(submission_material).map(|erc20_transfer_events| {
            let filtered = self
                .iter()
                .filter(|tx_info| {
                    let transfer_exists =
                        erc20_transfer_events.erc20_transfer_exists(&tx_info.to_erc20_transfer_event());
                    if transfer_exists {
                        true
                    } else {
                        warn!(
                            "✘ Tx info filtered out ∵ no ERC20 transfer event found for it! {:?}",
                            tx_info
                        );
                        false
                    }
                })
                .cloned()
                .collect::<Vec<BtcOnIntBtcTxInfo>>();
            info!("✔ Number of `BtcOnIntBtcTxInfos` after: {}", filtered.len());
            Self::new(filtered)
        })
    }
}

pub fn maybe_filter_those_with_no_corresponding_erc20_transfer_event<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out `BtcOnIntBtcTxInfos` which don't have corresponding ERC20 transfer events ...");
    state
        .eth_db_utils
        .get_eth_canon_block_from_db()
        .and_then(|canon_block_submission_material| {
            state
                .btc_on_int_btc_tx_infos
                .clone()
                .filter_those_with_no_corresponding_erc20_transfer_event(&canon_block_submission_material)
        })
        .and_then(|filtered| state.replace_btc_on_int_btc_tx_infos(filtered))
}
