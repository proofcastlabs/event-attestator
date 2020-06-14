use crate::{
    btc_on_eth::eth::{
        any_sender::relay_contract::RelayContract,
        eth_crypto::{eth_private_key::EthPrivateKey, eth_transaction::EthTransaction},
        eth_database_utils::{
            get_eth_chain_id_from_db, get_eth_private_key_from_db, get_public_eth_address_from_db,
        },
    },
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};
use ethabi::{encode, Token};
use ethereum_types::{Address as EthAddress, Signature as EthSignature};

const MAX_COMPENSATION_WEI: u64 = 50_000_000_000_000_000;

/// An any.sender relay transaction. It is very similar
/// to a normal transaction except for a few fields.
/// The schema can be found [here](https://github.com/PISAresearch/docs.any.sender/blob/master/docs/relayTx.schema.json).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct RelayTransaction {
    /// The standard eth chain id.
    /// Currently supports Ropsten = 3 and Mainnet = 1.
    chain_id: Byte,

    /// The ethereum address of the user
    /// authorising this relay transaction.
    pub from: EthAddress,

    /// A signature made by the `from` authority
    /// over the full relay transaction data.
    /// Using this [digest](https://github.com/PISAresearch/contracts.any.sender/blob/e7d9cf8c26bdcae67e39f464b4a102a8572ff468/versions/0.2.1/contracts/core/RelayTxStruct.sol#L22).
    pub signature: EthSignature,

    /// The ABI encoded call data.
    /// Same as standard Ethereum.
    /// Max data length is 3000 bytes (BETA).
    pub data: Bytes,

    /// The block by which this transaction must be mined.
    /// Must be at minimum 400 greater than current block (BETA).
    /// There is a tolerance of 20 blocks above and below 400 (BETA).
    /// Can optionally be set to 0. In this case the any.sender API will
    /// fill in a deadline (currentBlock + 400) and populate it in the returned receipt.
    // An integer in range 0..=9_007_199_254_740_991.
    pub deadline: u64,

    /// The gas limit provided to the transaction for execution.
    /// Same as standard Ethereum.
    /// An integer in range 0..=3.000.000 (BETA).
    pub gas_limit: u32,

    /// The value of the compensation that the user will be owed
    /// if any.sender fails to mine the transaction
    /// before the `deadline`.
    /// Max compensation is 0.05 ETH (BETA).
    // Maximum value 50_000_000_000_000_000
    pub compensation: u64,

    /// The address of the relay contract
    /// that will be used to relay this transaction.
    pub relay_contract_address: EthAddress,

    /// The address the transaction is directed to.
    /// Cannot be empty.
    pub to: EthAddress,
}

impl RelayTransaction {
    /// Creates a new signed relay transaction.
    pub fn new<D>(
        data: Bytes,
        deadline: Option<u64>,
        gas_limit: u32,
        compensation: u64,
        to: EthAddress,
        db: &D,
    ) -> Result<RelayTransaction>
    where
        D: DatabaseInterface,
    {
        let from = get_public_eth_address_from_db(db)?;
        let chain_id = get_eth_chain_id_from_db(db)?;
        let relay_contract_address = RelayContract::from_eth_chain_id(chain_id)?.address()?;
        let eth_private_key = get_eth_private_key_from_db(db)?;

        let relay_transaction = RelayTransaction::from_data_unsigned(
            chain_id,
            from,
            data,
            deadline,
            gas_limit,
            compensation,
            relay_contract_address,
            to,
        )?
        .sign(&eth_private_key)?;

        info!("✔ Any.sender transaction signature is calculated. Returning signed transaction...");

        Ok(relay_transaction)
    }

    /// Creates a new unsigned relay transaction from data.
    fn from_data_unsigned(
        chain_id: u8,
        from: EthAddress,
        data: Bytes,
        deadline: Option<u64>,
        gas_limit: u32,
        compensation: u64,
        relay_contract_address: EthAddress,
        to: EthAddress,
    ) -> Result<RelayTransaction> {
        info!("✔ Checking any.sender transaction constraints...");

        let deadline = deadline.unwrap_or_default();

        if deadline > 9_007_199_254_740_991 {
            return Err(AppError::Custom(
                "✘ Any.sender deadline is out of range!".to_string(),
            ));
        }

        if gas_limit > 3_000_000 {
            return Err(AppError::Custom(
                "✘ Any.sender gas limit is out of range!".to_string(),
            ));
        }

        if data.len() > 3_000 {
            return Err(AppError::Custom(
                "✘ Any.sender data length is out of range!".to_string(),
            ));
        }

        if compensation > MAX_COMPENSATION_WEI {
            return Err(AppError::Custom(
                "✘ Any.sender compensation cannot be grater than 0.05 ETH!".to_string(),
            ));
        }

        if chain_id != 1 && chain_id != 3 {
            return Err(AppError::Custom(
                "✘ Any.sender is not available on chain with the id provided!".to_string(),
            ));
        }

        info!(
            "✔ Any.sender transaction constraints are satisfied. Returning unsigned transaction..."
        );

        Ok(RelayTransaction {
            chain_id,
            from,
            data,
            deadline,
            gas_limit,
            compensation,
            relay_contract_address,
            to,
            signature: EthSignature::default(),
        })
    }

    /// Calculates any.sender relay transaction signature.
    fn sign(mut self, eth_private_key: &EthPrivateKey) -> Result<RelayTransaction> {
        info!("Calculating relay transaction signature...");

        let transaction_bytes = encode(&[
            Token::Address(self.to),
            Token::Address(self.from),
            Token::Bytes(self.data.clone()),
            Token::Uint(self.deadline.into()),
            Token::Uint(self.compensation.into()),
            Token::Uint(self.gas_limit.into()),
            Token::Uint(self.chain_id.into()),
            Token::Address(self.relay_contract_address),
        ]);

        let signed_message = eth_private_key.sign_eth_prefixed_msg_bytes(transaction_bytes)?;
        self.signature = EthSignature::from_slice(&signed_message);

        Ok(self)
    }

    /// Creates a new relay transaction from Ethereum transaction.
    pub fn from_eth_transaction<D>(
        eth_transaction: &EthTransaction,
        db: &D,
    ) -> Result<RelayTransaction>
    where
        D: DatabaseInterface,
    {
        let chain_id = eth_transaction.chain_id;
        let from = get_public_eth_address_from_db(db)?;
        let data = eth_transaction.data.clone();
        let deadline = None; // use the default any.sender deadline
        let gas_limit = eth_transaction.gas_limit.as_u32();
        let compensation = MAX_COMPENSATION_WEI;
        let relay_contract_address =
            RelayContract::from_eth_chain_id(eth_transaction.chain_id)?.address()?;
        let to = EthAddress::from_slice(&eth_transaction.to);

        let eth_private_key = get_eth_private_key_from_db(db)?;

        let relay_transaction = RelayTransaction::from_data_unsigned(
            chain_id,
            from,
            data,
            deadline,
            gas_limit,
            compensation,
            relay_contract_address,
            to,
        )?
        .sign(&eth_private_key)?;

        Ok(relay_transaction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::{
        eth::eth_database_utils::{
            put_eth_chain_id_in_db, put_eth_private_key_in_db, put_public_eth_address_in_db,
        },
        eth::eth_test_utils::get_sample_unsigned_eth_transaction,
        test_utils::{get_test_database, TestDB},
    };

    fn setup_db(recovery_param: Option<bool>) -> TestDB {
        let db = get_test_database();

        let chain_id = 3;
        put_eth_chain_id_in_db(&db, chain_id).expect("Error putting chain id in db!");

        let from = EthAddress::from_slice(
            &hex::decode(if recovery_param.unwrap_or_default() {
                "1a96829d85bdf719b58b2593e2853d4ae5a0f50b"
            } else {
                "736661736533BcfC9cc35649e6324aceFb7D32c1"
            })
            .unwrap(),
        );
        put_public_eth_address_in_db(&db, &from).expect("Error putting public eth address in db!");

        let eth_private_key = EthPrivateKey::from_slice(if recovery_param.unwrap_or_default() {
            [
                6, 55, 162, 221, 254, 198, 108, 20, 103, 12, 93, 123, 226, 232, 71, 70, 139, 212,
                41, 54, 65, 132, 18, 158, 202, 14, 137, 226, 174, 63, 11, 45,
            ]
        } else {
            [
                132, 23, 52, 203, 67, 154, 240, 53, 117, 195, 124, 41, 179, 50, 97, 159, 61, 169,
                234, 47, 186, 237, 88, 161, 200, 177, 24, 142, 207, 242, 168, 221,
            ]
        })
        .unwrap();
        put_eth_private_key_in_db(&db, &eth_private_key)
            .expect("Error putting eth private key in db!");

        db
    }

    #[test]
    fn should_create_new_signed_relay_tx_from_data() {
        let data = hex::decode("f15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000047465737400000000000000000000000000000000000000000000000000000000").unwrap();
        let deadline = Some(0);
        let gas_limit = 100000;
        let compensation = 500000000;
        let relay_contract_address = RelayContract::Ropsten.address().unwrap();
        let to = EthAddress::from_slice(
            &hex::decode("FDE83bd51bddAA39F15c1Bf50E222a7AE5831D83").unwrap(),
        );

        let expected_data = hex::decode("f15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000047465737400000000000000000000000000000000000000000000000000000000").unwrap();

        // private key without recovery param
        let from = EthAddress::from_slice(
            &hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap(),
        );

        let db = setup_db(None);

        let relay_transaction =
            RelayTransaction::new(data.clone(), deadline, gas_limit, compensation, to, &db)
                .unwrap();

        let expected_signature = EthSignature::from_slice(
            &hex::decode("5aa14a852439d9f5aa7b22c63a228d79c6822cf644badc9a63117dd7880d9a4c639eccd4aeeee91eaea63e36640d151be71346d785d2bd274fb82351c6bb2c101b")
                .unwrap(),
        );
        let expected_relay_transaction = RelayTransaction {
            signature: expected_signature,
            data: expected_data.clone(),
            chain_id: 3,
            deadline: 0,
            from,
            gas_limit,
            compensation,
            relay_contract_address,
            to,
        };

        assert_eq!(relay_transaction, expected_relay_transaction);

        // private key with recovery param
        let from = EthAddress::from_slice(
            &hex::decode("1a96829d85bdf719b58b2593e2853d4ae5a0f50b").unwrap(),
        );

        let db = setup_db(Some(true));

        let relay_transaction =
            RelayTransaction::new(data, deadline, gas_limit, compensation, to, &db).unwrap();

        let expected_signature = EthSignature::from_slice(
            &hex::decode("89397a8de1489ab225704fdfe2187a72d837659c190b6bd0c0e2b6cd5f2705da1fa1db87fd516f4677f6db821a6ede7b4f7f4779d9f248a7ed93c1b8ca86c48f1b")
                .unwrap()
            );
        let expected_relay_transaction = RelayTransaction {
            signature: expected_signature,
            data: expected_data,
            chain_id: 3,
            deadline: 0,
            from,
            gas_limit,
            compensation,
            relay_contract_address,
            to,
        };

        assert_eq!(relay_transaction, expected_relay_transaction);
    }

    #[test]
    fn should_create_new_signed_relay_tx_from_eth_tx() {
        let db = setup_db(None);

        let mut eth_transaction = get_sample_unsigned_eth_transaction();
        eth_transaction.chain_id = 3;

        let relay_transaction = RelayTransaction::from_eth_transaction(&eth_transaction, &db)
            .expect("Error creating any.sender relay transaction from eth transaction!");
        let expected_relay_transaction = RelayTransaction {
            chain_id: 3,
            from: EthAddress::from_slice(
                &hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap()),
            signature: EthSignature::from_slice(
                &hex::decode("7a8d98c902443d4035085b22af11869ff189eb2a1fd594eaba6335c7284a414d68cb6e9c44f0ff0b54b76a684072c0e7912d090894aa4f2b73dab19fd49cc90a1b").unwrap()),
            data: Bytes::default(),
            deadline: 0,
            gas_limit: 100000,
            compensation: 50000000000000000,
            relay_contract_address: EthAddress::from_slice(
                &hex::decode("9b4fa5a1d9f6812e2b56b36fbde62736fa82c2a7").unwrap()),
            to: EthAddress::from_slice(
                &hex::decode("53c2048dad4fcfab44c3ef3d16e882b5178df42b").unwrap()),
        };

        assert_eq!(relay_transaction, expected_relay_transaction);
    }
}
