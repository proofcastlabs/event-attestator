use common::types::Result;
use eos_on_int::CORE_TYPE;

make_cli_args_struct!(
    CORE_TYPE;
    flag_confs => u64,
    flag_version => bool,
    flag_gasPrice => u64,
    flag_chainId => String,
    flag_ethNetwork => String,
    flag_accountName => String,
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
    cmd_initializeInt => bool,
    cmd_initializeEos => bool,
    cmd_submitEosBlock => bool,
    cmd_submitIntBlock => bool,
    cmd_submitIntBlocks => bool,
    cmd_getEnclaveState => bool,
    cmd_signMessageWithIntKey => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_signHexMsgWithIntKeyWithPrefix => bool,
    cmd_signAsciiMsgWithIntKeyWithNoPrefix => bool,
    cmd_debugResetIntChain => bool,
    cmd_debugAddEosSchedule => bool,
    cmd_debugSetIntGasPrice => bool,
    cmd_debugReprocessIntBlock => bool,
    cmd_debugUpdateIncremerkle => bool,
    cmd_debugReprocessEosBlock => bool,
    cmd_debugSetIntAccountNonce => bool,
    cmd_debugSetEosAccountNonce => bool,
    cmd_debugAddDictionaryEntry => bool,
    cmd_debugRemoveDictionaryEntry => bool,
    cmd_debugEnableEosProtocolFeature => bool,
    cmd_debugDisableEosProtocolFeature => bool,
    cmd_debugReprocessEosBlockWithNonce => bool
);

impl CliArgs {
    pub fn maybe_set_dictionary_entry_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddDictionaryEntry {
            self.read_file_to_string().map(|s| self.update_arg_entry_json(s))
        } else {
            Ok(self)
        }
    }

    pub fn maybe_set_incremerkle_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugUpdateIncremerkle {
            self.read_file_to_string().map(|s| self.update_arg_eos_json(s))
        } else {
            Ok(self)
        }
    }

    pub fn maybe_set_block_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_initializeEos {
            self.read_file_to_string().map(|s| self.update_block_json(s))
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
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info)
        .and_then(CliArgs::maybe_set_block_json)
        .and_then(CliArgs::maybe_set_schedule_json)
        .and_then(CliArgs::maybe_set_incremerkle_json)
        .and_then(CliArgs::maybe_set_dictionary_entry_json)
}
