use common::types::Result;
use int_on_evm::CORE_TYPE;

make_cli_args_struct!(
    CORE_TYPE;
    flag_confs => u64,
    flag_chainId => u64,
    flag_gasPrice => u64,
    flag_version => bool,
    flag_ethNetwork => String,
    flag_vaultAddress => String,
    flag_routerAddress => String,
    arg_fee => u64,
    arg_nonce => u64,
    arg_key => String,
    arg_value => String,
    arg_gasPrice => u64,
    arg_amount => String,
    arg_message => String,
    arg_entryJson => String,
    arg_ethAddress => String,
    arg_tokenAddress => String,
    arg_vaultAddress => String,
    arg_routerAddress => String,
    arg_recipientAddress => String,
    cmd_initializeInt => bool,
    cmd_initializeEvm => bool,
    cmd_submitIntBlock => bool,
    cmd_submitEvmBlock => bool,
    cmd_submitIntBlocks => bool,
    cmd_submitEvmBlocks => bool,
    cmd_getEnclaveState => bool,
    cmd_debugResetIntChain => bool,
    cmd_debugResetEvmChain => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_debugSetFeeBasisPoints => bool,
    cmd_debugWithdrawFees => bool,
    cmd_debugSetIntGasPrice => bool,
    cmd_debugSetAccruedFees => bool,
    cmd_debugSetEvmGasPrice => bool,
    cmd_debugReprocessIntBlock => bool,
    cmd_debugReprocessEvmBlock => bool,
    cmd_debugAddSupportedToken => bool,
    cmd_debugSetIntAccountNonce => bool,
    cmd_debugSetEvmAccountNonce => bool,
    cmd_debugAddDictionaryEntry => bool,
    cmd_debugRemoveSupportedToken => bool,
    cmd_debugRemoveDictionaryEntry => bool,
    cmd_signHexMsgWithIntKeyWithPrefix => bool,
    cmd_signHexMsgWithEvmKeyWithPrefix => bool,
    cmd_debugReprocessIntBlockWithNonce => bool,
    cmd_debugReprocessEvmBlockWithNonce => bool,
    cmd_signAsciiMsgWithIntKeyWithNoPrefix => bool,
    cmd_signAsciiMsgWithEvmKeyWithNoPrefix => bool,
    cmd_debugReprocessIntBlockWithFeeAccrual => bool,
    cmd_debugReprocessEvmBlockWithFeeAccrual => bool
);

impl CliArgs {
    pub fn maybe_set_router_address_from_flag(self) -> Self {
        let value = self.flag_routerAddress.clone();
        if value.is_empty() {
            self
        } else {
            self.update_arg_router_address(value)
        }
    }

    pub fn maybe_set_vault_address_from_flag(self) -> Self {
        let value = self.flag_vaultAddress.clone();
        if value.is_empty() {
            self
        } else {
            self.update_arg_vault_address(value)
        }
    }

    pub fn maybe_set_entry_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddDictionaryEntry {
            self.read_file_to_string().map(|s| self.update_arg_entry_json(s))
        } else {
            Ok(self)
        }
    }
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info)
        .and_then(CliArgs::maybe_set_entry_json)
        .map(CliArgs::maybe_set_vault_address_from_flag)
        .map(CliArgs::maybe_set_router_address_from_flag)
}
