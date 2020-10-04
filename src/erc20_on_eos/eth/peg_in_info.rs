use ethereum_types::{
    U256,
    H256 as EthHash,
    Address as EthAddress,
};
use derive_more::{
    Deref,
    Constructor,
};
use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::{
        eos::eos_erc20_dictionary::EosErc20Dictionary,
        eth::{
            eth_state::EthState,
            eth_database_utils::get_eth_canon_block_from_db,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erc20OnEosPegInInfo {
    pub token_amount: U256,
    pub eos_address: String,
    pub account_name: String,
    pub eos_asset_amount: String,
    pub token_sender: EthAddress,
    pub token_contract: EthAddress,
    pub originating_tx_hash: EthHash,
}

impl Erc20OnEosPegInInfo {
    pub fn new(
        token_amount: U256,
        token_sender: EthAddress,
        token_contract: EthAddress,
        eos_address: String,
        originating_tx_hash: EthHash,
        account_name: String,
        eos_asset_amount: String,
    ) -> Erc20OnEosPegInInfo {
        Erc20OnEosPegInInfo {
            token_amount,
            token_contract,
            eos_address,
            originating_tx_hash,
            token_sender,
            account_name,
            eos_asset_amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct Erc20OnEosPegInInfos(pub Vec<Erc20OnEosPegInInfo>);

impl Erc20OnEosPegInInfos {
    pub fn sum(&self) -> U256 {
        self.0.iter().fold(U256::zero(), |acc, params| acc + params.token_amount)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn maybe_parse_peg_in_info_and_add_to_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe parsing `erc20-on-eos` peg-in infos...");
    get_eth_canon_block_from_db(&state.db)
        .and_then(|submission_material| {
            match submission_material.receipts.is_empty() {
                true => {
                    info!("✔ No receipts in canon block ∴ no info to parse!");
                    Ok(state)
                }
                false => {
                    info!("✔ {} receipts in canon block ∴ parsing info...", submission_material.block.number);
                    EosErc20Dictionary::get_from_db(&state.db)
                        .and_then(|account_names| submission_material.get_erc20_on_eos_peg_in_infos(&account_names))
                        .and_then(|peg_in_infos| state.add_erc20_on_eos_peg_in_infos(peg_in_infos))
                }
            }
        })
}
