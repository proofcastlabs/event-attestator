use lib::{btc_on_eos::CORE_TYPE, Result};

make_cli_args_struct!(
    CORE_TYPE;
    arg_fee => u64,
    arg_vOut => u32,
    arg_nonce => u64,
    arg_key => String,
    arg_txId => String,
    arg_value => String,
    arg_numUtxos => usize,
    arg_address => String,
    arg_eosJson => String,
    arg_basisPoints => u64,
    arg_utxosJson => String,
    arg_ethAddress => String,
    arg_accountName => String,
    arg_featureHash => String,
    arg_scheduleJson => String,
    flag_fee => u64,
    flag_confs => u64,
    flag_version => bool,
    flag_symbol => String,
    flag_difficulty => u64,
    flag_chainId => String,
    flag_network => String,
    cmd_initializeEos => bool,
    cmd_initializeBtc => bool,
    cmd_submitEosBlock => bool,
    cmd_submitBtcBlock => bool,
    cmd_getEnclaveState => bool,
    cmd_debugWithdrawFees => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_enableEosProtocolFeature => bool,
    cmd_disableEosProtocolFeature => bool,
    cmd_getAllUtxos => bool,
    cmd_debugAddUtxos => bool,
    cmd_debugSetBtcFee => bool,
    cmd_debugRemoveUtxo => bool,
    cmd_debugSetPegInFee => bool,
    cmd_debugSetPegOutFee => bool,
    cmd_debugAddEosSchedule => bool,
    cmd_debugConsolidateUtxos => bool,
    cmd_debugMaybeAddUtxoToDb => bool,
    cmd_debugReprocessBtcBlock => bool,
    cmd_debugReprocessEosBlock => bool,
    cmd_debugUpdateIncremerkle => bool,
    cmd_debugSetEosAccountNonce => bool,
    cmd_debugSetBtcAccountNonce => bool,
    cmd_debugGetChildPaysForParentTx => bool,
    cmd_debugConsolidateUtxosToAddress => bool,
    cmd_debugReprocessBtcBlockAccruingFees => bool,
    cmd_debugReprocessEosBlockAccruingFees => bool
);

impl CliArgs {
    pub fn maybe_set_eos_json(self) -> Result<Self> {
        if self.file_exists_at_path() && (self.cmd_initializeEos || self.cmd_debugUpdateIncremerkle) {
            self.read_file_to_string().map(|s| self.update_arg_eos_json(s))
        } else {
            Ok(self)
        }
    }

    pub fn maybe_set_eos_schedule(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddEosSchedule {
            self.read_file_to_string().map(|s| self.update_arg_schedule_json(s))
        } else {
            Ok(self)
        }
    }

    pub fn maybe_set_utxos_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddUtxos {
            self.read_file_to_string().map(|s| self.update_arg_utxos_json(s))
        } else {
            Ok(self)
        }
    }
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info)
        .and_then(CliArgs::maybe_set_eos_json)
        .and_then(CliArgs::maybe_set_utxos_json)
        .and_then(CliArgs::maybe_set_eos_schedule)
}
