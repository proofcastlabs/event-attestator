use std::str::FromStr;

use derive_more::{Constructor, Deref};
use eos_chain::AccountName as EosAccountName;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::{
        eos::{
            eos_chain_id::EosChainId,
            eos_crypto::{
                eos_private_key::EosPrivateKey,
                eos_transaction::{get_signed_eos_ptoken_issue_tx, EosSignedTransaction, EosSignedTransactions},
            },
            eos_utils::{get_eos_tx_expiration_timestamp_with_offset, remove_symbol_from_eos_asset},
        },
        eth::{
            eth_chain_id::EthChainId,
            eth_contracts::erc20_vault::{
                Erc20VaultPegInEventParams,
                ERC20_VAULT_PEG_IN_EVENT_WITHOUT_USER_DATA_TOPIC,
                ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
            },
            eth_database_utils::EthDbUtilsExt,
            eth_log::{EthLog, EthLogExt, EthLogs},
            eth_receipt::{EthReceipt, EthReceipts},
            eth_state::EthState,
            eth_submission_material::EthSubmissionMaterial,
        },
    },
    dictionaries::eos_eth::EosEthTokenDictionary,
    int_on_eos::int::eos_tx_info::{IntOnEosEosTxInfo, IntOnEosEosTxInfos},
    metadata::{
        metadata_address::MetadataAddress,
        metadata_chain_id::MetadataChainId,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::{ToMetadata, ToMetadataChainId},
        Metadata,
    },
    safe_addresses::safely_convert_str_to_eos_address,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

impl IntOnEosEosTxInfos {
    fn receipt_contains_relevant_logs(receipt: &EthReceipt, dictionary: &EosEthTokenDictionary) -> bool {
        Self::get_relevant_logs_from_receipt(receipt, dictionary).len() > 0
    }

    fn filter_eth_submission_material_for_supported_peg_ins(
        submission_material: &EthSubmissionMaterial,
        dictionary: &EosEthTokenDictionary,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts pertaining to `int-on-eos` peg-ins...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| Self::receipt_contains_relevant_logs(receipt, dictionary))
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        Ok(EthSubmissionMaterial::new(
            submission_material.get_block()?,
            filtered_receipts,
            submission_material.eos_ref_block_num,
            submission_material.eos_ref_block_prefix,
        ))
    }
}

pub fn filter_submission_material_for_relevant_receipts_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing relevant `INT-on-EOS` events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(
            &state.eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
            &[
                *ERC20_VAULT_PEG_IN_EVENT_WITHOUT_USER_DATA_TOPIC,
                *ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
            ],
        )
        .and_then(|filtered_submission_material| {
            IntOnEosEosTxInfos::filter_eth_submission_material_for_supported_peg_ins(
                &filtered_submission_material,
                state.get_eos_eth_token_dictionary()?,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}
