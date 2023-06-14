use common::{traits::DatabaseInterface, types::Result};
use common_btc::{
    convert_wei_to_satoshis,
    create_signed_raw_btc_tx_for_n_input_n_outputs,
    get_enough_utxos_to_cover_total,
    BtcPrivateKey,
    BtcRecipientAndAmount,
    BtcRecipientsAndAmounts,
    MAX_NUM_OUTPUTS,
};
use common_eth::{
    Erc777RedeemEvent,
    EthDbUtilsExt,
    EthLog,
    EthLogExt,
    EthReceipt,
    EthState,
    EthSubmissionMaterial,
    ERC777_REDEEM_EVENT_TOPIC_V2,
};
use common_safe_addresses::safely_convert_str_to_btc_address;
use ethereum_types::Address as EthAddress;

use crate::{
    bitcoin_crate_alias::blockdata::transaction::Transaction as BtcTransaction,
    int::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
};

impl BtcOnIntBtcTxInfo {
    pub fn to_btc_tx<D: DatabaseInterface>(
        &self,
        db: &D,
        fee: u64,
        btc_address: &str,
        btc_private_key: &BtcPrivateKey,
    ) -> Result<BtcTransaction> {
        let utxos = get_enough_utxos_to_cover_total(db, self.amount_in_satoshis, MAX_NUM_OUTPUTS, fee)?;
        info!("✔ Getting correct amount of UTXOs...");
        info!("✔ Satoshis per byte: {}", fee);
        info!("✔ Retrieved {} UTXOs!", utxos.len());
        info!("✔ Creating BTC transaction...");
        create_signed_raw_btc_tx_for_n_input_n_outputs(
            fee,
            BtcRecipientsAndAmounts::new(vec![self.to_recipient_and_amount()?]),
            btc_address,
            btc_private_key,
            utxos,
        )
    }

    pub fn to_recipient_and_amount(&self) -> Result<BtcRecipientAndAmount> {
        let recipient_and_amount = BtcRecipientAndAmount::new(&self.recipient[..], self.amount_in_satoshis);
        info!("✔ Recipient & amount retrieved from redeem: {:?}", recipient_and_amount);
        recipient_and_amount
    }
}

impl BtcOnIntBtcTxInfos {
    fn log_is_btc_on_int_redeem(log: &EthLog, erc777_smart_contract_address: &EthAddress) -> Result<bool> {
        Ok(log.is_from_address(erc777_smart_contract_address) && log.contains_topic(&ERC777_REDEEM_EVENT_TOPIC_V2))
    }

    fn from_eth_receipt(receipt: &EthReceipt, erc777_smart_contract_address: &EthAddress) -> Result<Self> {
        info!("✔ Getting redeem `BtcOnIntBtcTxInfos` from receipt...");
        Ok(Self::new(
            receipt
                .logs
                .0
                .iter()
                .filter(|log| {
                    matches!(
                        BtcOnIntBtcTxInfos::log_is_btc_on_int_redeem(log, erc777_smart_contract_address),
                        Ok(true)
                    )
                })
                .map(|log| {
                    let event_params = Erc777RedeemEvent::from_eth_log(log)?;
                    Ok(BtcOnIntBtcTxInfo {
                        to: EthAddress::zero(), // NOTE: Because this is a redeem, the token is burnt.
                        from: event_params.redeemer,
                        amount_in_wei: event_params.value,
                        token_address: *erc777_smart_contract_address,
                        originating_tx_hash: receipt.transaction_hash,
                        amount_in_satoshis: convert_wei_to_satoshis(event_params.value),
                        recipient: safely_convert_str_to_btc_address(&event_params.underlying_asset_recipient)
                            .to_string(),
                    })
                })
                .collect::<Result<Vec<BtcOnIntBtcTxInfo>>>()?,
        ))
    }

    pub fn from_eth_submission_material(
        submission_material: &EthSubmissionMaterial,
        erc777_smart_contract_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `BtcOnIntBtcTxInfos` from ETH submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Ok(Self::from_eth_receipt(receipt, erc777_smart_contract_address)?.0))
                .collect::<Result<Vec<Vec<BtcOnIntBtcTxInfo>>>>()?
                .concat(),
        ))
    }
}

pub fn maybe_parse_btc_on_int_tx_infos_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `BtcOnIntBtcTxInfos`...");
    state
        .eth_db_utils
        .get_eth_canon_block_from_db()
        .and_then(|submission_material| {
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in canon block ∴ no infos to parse!");
                Ok(state)
            } else {
                info!("✔ Receipts in canon block ∴ parsing infos...");
                BtcOnIntBtcTxInfos::from_eth_submission_material(
                    &submission_material,
                    &state.eth_db_utils.get_btc_on_int_smart_contract_address_from_db()?,
                )
                .and_then(|infos| infos.to_bytes())
                .map(|bytes| state.add_tx_infos(bytes))
            }
        })
}
