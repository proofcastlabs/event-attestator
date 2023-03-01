use common::types::Result;
use erc20_on_evm::CORE_TYPE;

make_cli_args_struct!(
    CORE_TYPE;
    flag_confs => u64,
    flag_chainId => u8,
    flag_gasPrice => u64,
    flag_version => bool,
    flag_ethNetwork => String,
    flag_vaultAddress => String,
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
    arg_recipientAddress => String,
    cmd_initializeEth => bool,
    cmd_initializeEvm => bool,
    cmd_submitEthBlock => bool,
    cmd_submitEvmBlock => bool,
    cmd_getEnclaveState => bool,
    cmd_debugResetEthChain => bool,
    cmd_debugResetEvmChain => bool,
    cmd_getLatestBlockNumbers => bool,
    cmd_debugSetFeeBasisPoints => bool,
    cmd_addVaultContractAddress => bool,
    cmd_debugWithdrawFees => bool,
    cmd_debugMigrateSingle => bool,
    cmd_debugSetEthGasPrice => bool,
    cmd_debugSetAccruedFees => bool,
    cmd_debugSetEvmGasPrice => bool,
    cmd_debugMigrateContract => bool,
    cmd_debugReprocessEthBlock => bool,
    cmd_debugReprocessEvmBlock => bool,
    cmd_debugAddSupportedToken => bool,
    cmd_debugSetEthAccountNonce => bool,
    cmd_debugSetEvmAccountNonce => bool,
    cmd_debugAddDictionaryEntry => bool,
    cmd_debugRemoveSupportedToken => bool,
    cmd_debugRemoveDictionaryEntry => bool,
    cmd_signHexMsgWithEthKeyWithPrefix => bool,
    cmd_signHexMsgWithEvmKeyWithPrefix => bool,
    cmd_debugReprocessEthBlockWithNonce => bool,
    cmd_debugReprocessEvmBlockWithNonce => bool,
    cmd_signAsciiMsgWithEthKeyWithNoPrefix => bool,
    cmd_signAsciiMsgWithEvmKeyWithNoPrefix => bool,
    cmd_debugReprocessEthBlockWithFeeAccrual => bool,
    cmd_debugReprocessEvmBlockWithFeeAccrual => bool
);

impl CliArgs {
    fn maybe_set_dictionary_entry_json(self) -> Result<Self> {
        if self.file_exists_at_path() && self.cmd_debugAddDictionaryEntry {
            self.read_file_to_string().map(|s| self.update_arg_entry_json(s))
        } else {
            Ok(self)
        }
    }
}

pub fn get_cli_args(usage_info: &str) -> Result<CliArgs> {
    CliArgs::parse(usage_info).and_then(CliArgs::maybe_set_dictionary_entry_json)
}
