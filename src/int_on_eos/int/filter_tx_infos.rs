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
    int_on_eos::int::eos_tx_info::{IntOnEosEosTxInfos, IntOnEosEosTxInfo},
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

impl IntOnEosEosTxInfo {
    pub fn is_zero_eos_amount(&self, dictionary: &EosEthTokenDictionary) -> Result<bool> {
        let entry = dictionary.get_entry_via_eos_address(&EosAccountName::from_str(&self.eos_token_address)?)?;
        let eos_amount = remove_symbol_from_eos_asset(&entry.convert_u256_to_eos_asset_string(&self.token_amount)?)
            .parse::<f64>()?;
        Ok(eos_amount == 0.0)
    }
}

impl IntOnEosEosTxInfos {
    pub fn filter_out_zero_eos_values(&self, dictionary: &EosEthTokenDictionary) -> Result<Self> {
        Ok(Self::new(
            self.iter()
                .map(|tx_info| {
                    let is_zero_eos_amount = tx_info.is_zero_eos_amount(dictionary)?;
                    Ok((tx_info.clone(), is_zero_eos_amount))
                })
                .collect::<Result<Vec<(IntOnEosEosTxInfo, bool)>>>()?
                .iter()
                .filter_map(|(tx_info, is_zero_amount)| {
                    if *is_zero_amount {
                        info!(
                            "✘ Filtering out peg in info due to zero EOS asset amount: {:?}",
                            tx_info
                        );
                        None
                    } else {
                        Some(tx_info)
                    }
                })
                .cloned()
                .collect::<Vec<IntOnEosEosTxInfo>>(),
        ))
    }
}

pub fn filter_out_zero_value_eos_tx_infos_from_state<D: DatabaseInterface>(
    state: EthState<D>
) -> Result<EthState<D>> {
    info!("✔ Maybe filtering `INT-on-EOS` EOS tx infos...");
    debug!("✔ Num infos before: {}", state.int_on_eos_eos_tx_infos.len());
    state
        .int_on_eos_eos_tx_infos
        .filter_out_zero_eos_values(&EosEthTokenDictionary::get_from_db(state.db)?)
        .and_then(|filtered_infos| {
            debug!("✔ Num infos after: {}", filtered_infos.len());
            state.replace_int_on_eos_eos_tx_infos(filtered_infos)
        })
}
