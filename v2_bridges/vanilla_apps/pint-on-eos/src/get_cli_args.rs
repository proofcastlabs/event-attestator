use common::types::Result;
use int_on_eos::CORE_TYPE;

make_cli_args_struct!(
    CORE_TYPE;
    flag_confs => u64,
    flag_gasPrice => u64,
    flag_version => bool,
    flag_chainId => String,
    flag_vaultAddress => String,
    flag_routerAddress => String,
    arg_wei => u64,
    arg_nonce => u64,
    arg_key => String,
    arg_value => String,
    arg_message => String,
    arg_eosJson => String,
    arg_entryJson => String,
    arg_ethAddress => String,
    arg_featureHash => String,
    arg_scheduleJson => String,
    cmd_initializeEos => bool,
    cmd_initializeInt => bool,
    cmd_submitEosBlock => bool,
    cmd_submitIntBlock => bool,
    cmd_submitIntBlocks => bool,
    cmd_getEnclaveState => bool,
    cmd_debugResetIntChain => bool,
    cmd_debugSetIntGasPrice => bool,
    cmd_debugAddEosSchedule => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_debugReprocessIntBlock => bool,
    cmd_debugReprocessEosBlock => bool,
    cmd_debugUpdateIncremerkle => bool,
    cmd_debugAddSupportedToken => bool,
    cmd_debugSetIntAccountNonce => bool,
    cmd_debugAddDictionaryEntry => bool,
    cmd_debugSetEosAccountNonce => bool,
    cmd_debugRemoveSupportedToken => bool,
    cmd_debugRemoveDictionaryEntry => bool,
    cmd_debugEnableEosProtocolFeature => bool,
    cmd_signHexMsgWithIntKeyWithPrefix => bool,
    cmd_debugDisableEosProtocolFeature => bool,
    cmd_debugReprocessEosBlockWithNonce => bool,
    cmd_signAsciiMsgWithIntKeyWithNoPrefix => bool
);

impl CliArgs {
    pub fn maybe_set_dictionary_entry_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddDictionaryEntry {
            self.read_file_to_string().map(|s| self.update_arg_entry_json(s))
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

    pub fn maybe_set_eos_schedule(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddEosSchedule {
            self.read_file_to_string().map(|s| self.update_arg_schedule_json(s))
        } else {
            Ok(self)
        }
    }
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info)
        .and_then(CliArgs::maybe_set_eos_json)
        .and_then(CliArgs::maybe_set_dictionary_entry_json)
        .and_then(CliArgs::maybe_set_eos_schedule)
}
