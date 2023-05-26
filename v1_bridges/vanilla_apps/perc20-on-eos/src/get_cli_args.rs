use common::types::Result;
use erc20_on_eos::CORE_TYPE;

make_cli_args_struct!(
    CORE_TYPE;
    flag_confs => u64,
    flag_gasPrice => u64,
    flag_version => bool,
    flag_chainId => String,
    flag_vaultAddress => String,
    arg_wei => u64,
    arg_nonce => u64,
    arg_key => String,
    arg_value => String,
    arg_amount => String,
    arg_message => String,
    arg_address => String,
    arg_eosJson => String,
    arg_basisPoints => u64,
    arg_entryJson => String,
    arg_ethAddress => String,
    arg_featureHash => String,
    arg_scheduleJson => String,
    arg_tokenAddress => String,
    arg_recipientAddress => String,
    cmd_initializeEos => bool,
    cmd_initializeEth => bool,
    cmd_submitEosBlock => bool,
    cmd_submitEthBlock => bool,
    cmd_getEnclaveState => bool,
    cmd_debugResetEthChain => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_addVaultContractAddress => bool,
    cmd_enableEosProtocolFeature => bool,
    cmd_disableEosProtocolFeature => bool,
    cmd_debugWithdrawFees => bool,
    cmd_debugMigrateSingle => bool,
    cmd_debugSetEthGasPrice => bool,
    cmd_debugAddEosSchedule => bool,
    cmd_debugSetAccruedFees => bool,
    cmd_debugMigrateContract => bool,
    cmd_debugReprocessEthBlock => bool,
    cmd_debugReprocessEosBlock => bool,
    cmd_debugUpdateIncremerkle => bool,
    cmd_debugAddSupportedToken => bool,
    cmd_debugSetEthAccountNonce => bool,
    cmd_debugAddDictionaryEntry => bool,
    cmd_debugSetEosAccountNonce => bool,
    cmd_debugRemoveSupportedToken => bool,
    cmd_debugSetEthFeeBasisPoints => bool,
    cmd_debugSetEosFeeBasisPoints => bool,
    cmd_debugRemoveDictionaryEntry => bool,
    cmd_signMessageWithEthKey => bool,
    cmd_signHexMsgWithEthKeyWithPrefix => bool,
    cmd_debugReprocessEosBlockWithNonce => bool,
    cmd_signAsciiMsgWithEthKeyWithNoPrefix => bool,
    cmd_debugReprocessEthBlockWithFeeAccrual => bool,
    cmd_debugReprocessEosBlockWithFeeAccrual => bool
);

impl CliArgs {
    pub fn maybe_set_dictionary_entry_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddDictionaryEntry {
            self.read_file_to_string().map(|s| self.update_arg_entry_json(s))
        } else {
            Ok(self)
        }
    }

    pub fn maybe_set_schedule_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddEosSchedule {
            self.read_file_to_string().map(|s| self.update_arg_schedule_json(s))
        } else {
            Ok(self)
        }
    }

    pub fn maybe_set_eos_json(self) -> Result<Self> {
        if self.file_exists_at_path() && (self.cmd_initializeEos || self.cmd_debugUpdateIncremerkle) {
            self.read_file_to_string().map(|s| self.update_arg_eos_json(s))
        } else {
            Ok(self)
        }
    }
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info)
        .and_then(CliArgs::maybe_set_eos_json)
        .and_then(CliArgs::maybe_set_schedule_json)
        .and_then(CliArgs::maybe_set_dictionary_entry_json)
}
