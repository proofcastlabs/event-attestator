pub use bitcoin::blockdata::transaction::Transaction as BtcTransaction;

use crate::{
    btc_on_eos::BtcOnEosBtcTxInfos,
    chains::{
        btc::{btc_database_utils::BtcDbUtils, utxo_manager::utxo_types::BtcUtxosAndValues},
        eos::{
            eos_action_proofs::EosActionProofs,
            eos_block_header::EosBlockHeaderV2,
            eos_database_utils::EosDbUtils,
            eos_global_sequences::{GlobalSequences, ProcessedGlobalSequences},
            eos_merkle_utils::Incremerkle,
            eos_producer_schedule::EosProducerScheduleV2,
            eos_submission_material::EosSubmissionMaterial,
            eos_types::Checksum256s,
            protocol_features::EnabledFeatures,
        },
        eth::{eth_crypto::eth_transaction::EthTransactions, eth_database_utils::EthDbUtils},
    },
    dictionaries::eos_eth::EosEthTokenDictionary,
    eos_on_eth::EosOnEthEosTxInfos,
    eos_on_int::EosOnIntIntTxInfos,
    erc20_on_eos::Erc20OnEosEthTxInfos,
    int_on_eos::IntOnEosIntTxInfos,
    traits::DatabaseInterface,
    types::Result,
    utils::get_not_in_state_err,
};

make_state_setters_and_getters!(
    EosState;
    "active_schedule" => EosProducerScheduleV2,
    "btc_utxos_and_values" => BtcUtxosAndValues,
    "eos_eth_token_dictionary" => EosEthTokenDictionary
);

macro_rules! impl_tx_info_fxns {
    ($($tx_info:ident),* $(,)?) => {
        paste! {
            impl<'a, D: DatabaseInterface> EosState<'a, D> {
                fn get_already_in_state_err_msg(thing: &str) -> String {
                    format!("'{}' is already in EOS state - not overwriting it!", thing)
                }

                $(
                    fn [< update_ $tx_info:snake >](mut self, infos: $tx_info) -> Result<Self> {
                        info!("✔ Updating `{}` in state...", stringify!($tx_info));
                        self.global_sequences = infos.get_global_sequences();
                        self.[< $tx_info:snake >] = infos;
                        Ok(self)
                    }

                    pub fn [< add_ $tx_info:snake >](self, infos: $tx_info) -> Result<Self> {
                        if self.[< $tx_info:snake >].is_empty() {
                            self.[< update_ $tx_info:snake >](infos)
                        } else {
                            Err(Self::get_already_in_state_err_msg(stringify!($tx_info)).into())
                        }
                    }

                    pub fn [< replace_ $tx_info:snake >](self, infos: $tx_info) -> Result<Self> {
                        info!("✔ Replacing `{}` in state...", stringify!($tx_info));
                        self.[< update_ $tx_info:snake >](infos)
                    }
                )*
            }
        }
    }
}

impl_tx_info_fxns!(
    BtcOnEosBtcTxInfos,
    EosOnEthEosTxInfos,
    EosOnIntIntTxInfos,
    IntOnEosIntTxInfos,
    Erc20OnEosEthTxInfos,
);

#[derive(Clone, Debug, PartialEq)]
pub struct EosState<'a, D: DatabaseInterface> {
    pub db: &'a D,
    pub block_num: Option<u64>,
    pub incremerkle: Incremerkle,
    pub producer_signature: String,
    global_sequences: GlobalSequences,
    pub action_proofs: EosActionProofs,
    pub interim_block_ids: Checksum256s,
    pub eth_signed_txs: EthTransactions,
    pub eth_db_utils: EthDbUtils<'a, D>,
    pub btc_db_utils: BtcDbUtils<'a, D>,
    pub eos_db_utils: EosDbUtils<'a, D>,
    pub block_header: Option<EosBlockHeaderV2>,
    pub btc_on_eos_signed_txs: Vec<BtcTransaction>,
    pub processed_tx_ids: ProcessedGlobalSequences,
    pub enabled_protocol_features: EnabledFeatures,
    pub int_on_eos_int_tx_infos: IntOnEosIntTxInfos,
    pub eos_on_int_int_tx_infos: EosOnIntIntTxInfos,
    pub eos_on_eth_eos_tx_infos: EosOnEthEosTxInfos,
    pub btc_on_eos_btc_tx_infos: BtcOnEosBtcTxInfos,
    pub active_schedule: Option<EosProducerScheduleV2>,
    pub btc_utxos_and_values: Option<BtcUtxosAndValues>,
    pub erc20_on_eos_eth_tx_infos: Erc20OnEosEthTxInfos,
    eos_eth_token_dictionary: Option<EosEthTokenDictionary>,
}

impl<'a, D: DatabaseInterface> EosState<'a, D> {
    pub fn init(db: &'a D) -> EosState<'a, D> {
        EosState {
            db,
            block_num: None,
            block_header: None,
            action_proofs: vec![],
            active_schedule: None,
            interim_block_ids: vec![],
            btc_utxos_and_values: None,
            btc_on_eos_signed_txs: vec![],
            eos_eth_token_dictionary: None,
            btc_db_utils: BtcDbUtils::new(db),
            eth_db_utils: EthDbUtils::new(db),
            producer_signature: String::new(),
            incremerkle: Incremerkle::default(),
            eos_db_utils: EosDbUtils::new(db),
            global_sequences: GlobalSequences::default(),
            eth_signed_txs: EthTransactions::new(vec![]),
            enabled_protocol_features: EnabledFeatures::init(),
            processed_tx_ids: ProcessedGlobalSequences::new(vec![]),
            eos_on_eth_eos_tx_infos: EosOnEthEosTxInfos::new(vec![]),
            int_on_eos_int_tx_infos: IntOnEosIntTxInfos::new(vec![]),
            eos_on_int_int_tx_infos: EosOnIntIntTxInfos::new(vec![]),
            btc_on_eos_btc_tx_infos: BtcOnEosBtcTxInfos::new(vec![]),
            erc20_on_eos_eth_tx_infos: Erc20OnEosEthTxInfos::new(vec![]),
        }
    }

    pub fn add_btc_on_eos_signed_txs(mut self, btc_on_eos_signed_txs: Vec<BtcTransaction>) -> Result<EosState<'a, D>> {
        self.btc_on_eos_signed_txs = btc_on_eos_signed_txs;
        Ok(self)
    }

    pub fn add_eth_signed_txs(mut self, txs: EthTransactions) -> Result<EosState<'a, D>> {
        self.eth_signed_txs = txs;
        Ok(self)
    }

    pub fn add_incremerkle(mut self, incremerkle: Incremerkle) -> EosState<'a, D> {
        self.incremerkle = incremerkle;
        self
    }

    pub fn add_submission_material(mut self, submission_material: EosSubmissionMaterial) -> Result<EosState<'a, D>> {
        self.block_num = Some(submission_material.block_num);
        self.action_proofs = submission_material.action_proofs;
        self.block_header = Some(submission_material.block_header);
        self.interim_block_ids = submission_material.interim_block_ids;
        self.producer_signature = submission_material.producer_signature;
        Ok(self)
    }

    pub fn add_processed_tx_ids(mut self, tx_ids: ProcessedGlobalSequences) -> Result<Self> {
        self.processed_tx_ids = tx_ids;
        Ok(self)
    }

    pub fn add_enabled_protocol_features(mut self, enabled_protocol_features: EnabledFeatures) -> Result<Self> {
        self.enabled_protocol_features = enabled_protocol_features;
        Ok(self)
    }

    pub fn get_eos_block_header(&self) -> Result<&EosBlockHeaderV2> {
        match self.block_header {
            Some(ref block_header) => Ok(block_header),
            None => Err(get_not_in_state_err("block_header").into()),
        }
    }

    pub fn get_eos_block_num(&self) -> Result<u64> {
        match self.block_num {
            Some(num) => Ok(num),
            None => Err(get_not_in_state_err("block_num").into()),
        }
    }

    pub fn replace_btc_on_eos_signed_txs(mut self, replacements: Vec<BtcTransaction>) -> Self {
        info!("✔ Replacing signed BTC txs infos in state...");
        self.btc_on_eos_signed_txs = replacements;
        self
    }

    pub fn replace_action_proofs(mut self, replacements: EosActionProofs) -> Result<EosState<'a, D>> {
        info!("✔ Replacing `action_proofs` in state...");
        self.action_proofs = replacements;
        Ok(self)
    }

    pub fn get_global_sequences(&self) -> GlobalSequences {
        self.global_sequences.clone()
    }
}
