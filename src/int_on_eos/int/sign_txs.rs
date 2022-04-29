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
    pub fn to_eos_signed_txs(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        private_key: &EosPrivateKey,
        dictionary: &EosEthTokenDictionary,
    ) -> Result<EosSignedTransactions> {
        info!("✔ Signing {} EOS txs from `erc20-on-eos` peg in infos...", self.len());
        Ok(EosSignedTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, info)| {
                    info.to_eos_signed_tx(
                        ref_block_num,
                        ref_block_prefix,
                        chain_id,
                        private_key,
                        get_eos_tx_expiration_timestamp_with_offset(i as u32)?,
                        dictionary,
                    )
                })
                .collect::<Result<Vec<EosSignedTransaction>>>()?,
        ))
    }
}

impl IntOnEosEosTxInfo {
    pub fn to_eos_signed_tx(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        private_key: &EosPrivateKey,
        timestamp: u32,
        dictionary: &EosEthTokenDictionary,
    ) -> Result<EosSignedTransaction> {
        info!("✔ Signing EOS tx from `IntOnEosEosTxInfo`: {:?}", self);
        let dictionary_entry =
            dictionary.get_entry_via_eos_address(&EosAccountName::from_str(&self.eos_token_address)?)?;
        let eos_amount = dictionary_entry.convert_u256_to_eos_asset_string(&self.token_amount)?;
        get_signed_eos_ptoken_issue_tx(
            ref_block_num,
            ref_block_prefix,
            &self.destination_address,
            &eos_amount,
            chain_id,
            private_key,
            &self.eos_token_address,
            timestamp,
            if self.user_data.is_empty() {
                None
            } else {
                info!("✔ Wrapping `user_data` in metadata for `IntOnEosEosTxInfo¬");
                Some(
                    self.to_metadata()?
                        .to_bytes_for_protocol(&chain_id.to_metadata_chain_id().to_protocol_id())?,
                )
            },
        )
    }
}

pub fn maybe_sign_eos_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe signing `INT-on-EOS` EOS txs...");
    let submission_material = state.get_eth_submission_material()?;
    state
        .int_on_eos_eos_tx_infos
        .to_eos_signed_txs(
            submission_material.get_eos_ref_block_num()?,
            submission_material.get_eos_ref_block_prefix()?,
            &state.eos_db_utils.get_eos_chain_id_from_db()?,
            &EosPrivateKey::get_from_db(state.db)?,
            &EosEthTokenDictionary::get_from_db(state.db)?,
        )
        .and_then(|signed_txs| state.add_eos_transactions(signed_txs))
}
