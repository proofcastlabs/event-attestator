use crate::{
    btc_on_eth::eth::{
        any_sender::relay_contract::RelayContract,
        eth_crypto::{eth_private_key::EthPrivateKey, eth_transaction::EthTransaction},
        eth_database_utils::{
            get_eth_chain_id_from_db, get_eth_private_key_from_db, get_latest_eth_block_number,
            get_public_eth_address_from_db,
        },
    },
    errors::AppError,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};
use ethabi::{encode, Token};
use ethereum_types::{Address as EthAddress, Signature as EthSignature};
use tiny_keccak::keccak256;

const MAX_COMPENSATION_WEI: u64 = 50_000_000_000_000_000;

/// An any.sender relay transaction. It is very similar
/// to a normal transaction except for a few fields.
/// The schema can be found [here](https://github.com/PISAresearch/docs.any.sender/blob/master/docs/relayTx.schema.json).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct RelayTransaction {
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
    // An integer in range 0..=9_007_199_254_740_991.
    pub deadline_block_number: u64,

    /// The gas limit provided to the transaction for execution.
    /// Same as standard Ethereum.
    /// An integer in range 0..=3.000.000 (BETA).
    pub gas: u32,

    /// The value of the compensation that the user will be owed
    /// if any.sender fails to mine the transaction
    /// before the `deadline_block_number`.
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
        deadline_block_number: u64,
        gas: u32,
        compensation: u64,
        to: EthAddress,
        db: &D,
    ) -> Result<RelayTransaction>
    where
        D: DatabaseInterface,
    {
        let from = get_public_eth_address_from_db(db)?;

        let eth_chain_id = get_eth_chain_id_from_db(db)?;
        let relay_contract_address =
            EthAddress::from(RelayContract::from_eth_chain_id(eth_chain_id)?);

        let eth_private_key = get_eth_private_key_from_db(db)?;

        let relay_transaction = RelayTransaction::from_data_unsigned(
            from,
            data,
            deadline_block_number,
            gas,
            compensation,
            relay_contract_address,
            to,
            db,
        )?
        .sign(&eth_private_key)?;

        info!("✔ Any.sender transaction signature is calculated. Returning signed transaction...");

        Ok(relay_transaction)
    }

    /// Creates a new unsigned relay transaction from data.
    fn from_data_unsigned<D>(
        from: EthAddress,
        data: Bytes,
        deadline_block_number: u64,
        gas: u32,
        compensation: u64,
        relay_contract_address: EthAddress,
        to: EthAddress,
        db: &D,
    ) -> Result<RelayTransaction>
    where
        D: DatabaseInterface,
    {
        info!("✔ Checking any.sender transaction constraints...");

        let minimum_deadline = get_latest_eth_block_number(db)? as u64 + 400;

        if !(minimum_deadline..=9_007_199_254_740_991).contains(&deadline_block_number) {
            return Err(AppError::Custom(
                "✘ Any.sender deadline_block_number is out of range!".to_string(),
            ));
        }

        if !(0..=3_000_000).contains(&gas) {
            return Err(AppError::Custom(
                "✘ Any.sender gas is out of range!".to_string(),
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

        info!(
            "✔ Any.sender transaction constraints are satisfied. Returning unsigned transaction..."
        );

        Ok(RelayTransaction {
            from,
            data,
            deadline_block_number,
            gas,
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
            Token::Uint(self.deadline_block_number.into()),
            Token::Uint(self.compensation.into()),
            Token::Uint(self.gas.into()),
            Token::Address(self.relay_contract_address),
        ]);
        let relay_tx_id = keccak256(&transaction_bytes);

        let message_bytes: Vec<u8> = [
            b"\x19Ethereum Signed Message:\n",
            relay_tx_id.len().to_string().as_bytes(),
            &relay_tx_id,
        ]
        .concat();

        let mut signed_message = eth_private_key.sign_message_bytes(message_bytes)?;

        // change last byte due to recovery param presence
        signed_message[64] = 28; // 0x1c

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
        let from = get_public_eth_address_from_db(db)?;

        let data = eth_transaction.data.clone();
        let deadline_block_number = get_latest_eth_block_number(db)? as u64 + 405;
        let gas = eth_transaction.gas_limit.as_u32();
        let compensation = MAX_COMPENSATION_WEI;
        let relay_contract_address =
            EthAddress::from(RelayContract::from_eth_chain_id(eth_transaction.chain_id)?);
        let to = EthAddress::from_slice(&eth_transaction.to);

        let eth_private_key = get_eth_private_key_from_db(db)?;

        let relay_transaction = RelayTransaction::from_data_unsigned(
            from,
            data,
            deadline_block_number,
            gas,
            compensation,
            relay_contract_address,
            to,
            db,
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
            put_special_eth_block_in_db,
        },
        eth::eth_test_utils::{
            get_sample_eth_block_and_receipts_n, get_sample_unsigned_eth_transaction,
        },
        test_utils::{get_test_database, TestDB},
    };

    fn setup_db() -> TestDB {
        let db = get_test_database();

        let chain_id = 3;
        put_eth_chain_id_in_db(&db, chain_id).expect("Error putting chain id in db!");

        let from = EthAddress::from_slice(
            &hex::decode("0590c44fc2d5971bca9407399d65144f97de6e12").unwrap(),
        );
        put_public_eth_address_in_db(&db, &from).expect("Error putting public eth address in db!");

        let block_type = "latest";
        let block = get_sample_eth_block_and_receipts_n(1).unwrap();
        put_special_eth_block_in_db(&db, &block, &block_type)
            .expect("Error putting ETH special block in db!");

        let eth_private_key = EthPrivateKey::from_slice([
            94, 198, 246, 91, 76, 118, 240, 238, 182, 141, 19, 140, 15, 63, 112, 18, 212, 176, 49,
            147, 40, 163, 118, 50, 200, 8, 193, 250, 236, 16, 135, 82,
        ])
        .unwrap();
        put_eth_private_key_in_db(&db, &eth_private_key)
            .expect("Error putting eth private key in db!");

        db
    }

    #[test]
    fn should_create_new_signed_relay_tx_from_data() {
        let data = hex::decode("f15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000992d2d204d792074657374206f66206563686f203a29202d2d20286d6573736167652073656e742062792030783035393063343466433264353937316263413934303733393944363531343446393764653665313220617420546875204d617920323120323032302031383a33333a323920474d542b30323030202843656e7472616c204575726f7065616e2053756d6d65722054696d65292900000000000000").unwrap();
        let deadline_block_number = 7945019;
        let gas = 100000;
        let compensation = 500000000;
        let relay_contract_address = EthAddress::from(RelayContract::Ropsten);
        let to = EthAddress::from_slice(
            &hex::decode("FDE83bd51bddAA39F15c1Bf50E222a7AE5831D83").unwrap(),
        );
        let from = EthAddress::from_slice(
            &hex::decode("0590c44fc2d5971bca9407399d65144f97de6e12").unwrap(),
        );

        let db = setup_db();

        let relay_transaction =
            RelayTransaction::new(data, deadline_block_number, gas, compensation, to, &db).unwrap();

        let expected_data = hex::decode("f15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000992d2d204d792074657374206f66206563686f203a29202d2d20286d6573736167652073656e742062792030783035393063343466433264353937316263413934303733393944363531343446393764653665313220617420546875204d617920323120323032302031383a33333a323920474d542b30323030202843656e7472616c204575726f7065616e2053756d6d65722054696d65292900000000000000").unwrap();
        let expected_signature = EthSignature::from_slice(
            &hex::decode("11346dc52736f16c31a19cc5cf9a99c8084f0802976f42a437d2380ab06cec463f257468a62294e24067686d22c17be2addbcd3a5b9abea15ac3937f32dba1341c")
                .unwrap(),
        );
        let expected_relay_transaction = RelayTransaction {
            signature: expected_signature,
            data: expected_data,
            from,
            deadline_block_number,
            gas,
            compensation,
            relay_contract_address,
            to,
        };

        assert_eq!(relay_transaction, expected_relay_transaction);
    }

    #[test]
    fn should_create_new_signed_relay_tx_from_eth_tx() {
        let db = setup_db();

        let mut eth_transaction = get_sample_unsigned_eth_transaction();
        eth_transaction.chain_id = 1;

        let relay_transaction = RelayTransaction::from_eth_transaction(&eth_transaction, &db)
            .expect("Error creating any.sender relay transaction from eth transaction!");
        let expected_relay_transaction = RelayTransaction {
            from: EthAddress::from_slice(
                &hex::decode("0590c44fc2d5971bca9407399d65144f97de6e12").unwrap()),
            signature: EthSignature::from_slice(
                &hex::decode("d31dbb91be068573abf2207312608cf41f7fdbd529a3a2be32f18c675be6ace21abb8b220aa11940c36b1b4100ead0ccd2b839706e574ffe44c949bb36cfe2111c").unwrap()),
            data: Bytes::default(),
            deadline_block_number: 7004991,
            gas: 100000,
            compensation: 50000000000000000,
            relay_contract_address: EthAddress::from_slice(
                &hex::decode("a404d1219ed6fe3cf2496534de2af3ca17114b06").unwrap()),
            to: EthAddress::from_slice(
                &hex::decode("53c2048dad4fcfab44c3ef3d16e882b5178df42b").unwrap()),
        };

        assert_eq!(relay_transaction, expected_relay_transaction);
    }
}
