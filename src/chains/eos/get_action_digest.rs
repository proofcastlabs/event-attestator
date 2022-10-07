use bitcoin::hashes::{sha256, Hash};
use eos_chain::{Action as EosAction, Digest, NumBytes, SerializeData, UnsignedInt, Write};

use crate::types::{Bytes, Result};

pub fn get_action_digest(action: &EosAction, action_has_return_value: bool) -> Result<Bytes> {
    if !action_has_return_value {
        debug!("Using original way to calculate action digest...");
        let digest = action.digest()?.as_bytes().to_vec();
        debug!("Action digest: 0x{}", hex::encode(&digest));
        Ok(digest)
    } else {
        debug!("Using `action_return_value` protocol feature to calculate action digest...");
        let serialized_action = action.to_serialize_data()?;
        let hash_1 = sha256::Hash::hash(&serialized_action[..33]).to_vec();
        let data_length = action.data.len();
        #[rustfmt::skip]
        let hash_2 = sha256::Hash::hash(&[
            bitpack_length(data_length)?,
            action.data.clone(),
            // NOTE: The final 0x00 is actually the length of the action return data, in our case zero.
            // If we support the action return data too, we'll need to include it and its length here.
            vec![0x00]
        ].concat()).to_vec();
        let digest = sha256::Hash::hash(&vec![hash_1, hash_2].concat()).to_vec();
        debug!("Action digest: 0x{}", hex::encode(&digest));
        Ok(digest)
    }
}

fn bitpack_length(data_length: usize) -> Result<Bytes> {
    let unsigned_int = UnsignedInt::from(data_length);
    // NOTE: Arbritrary length here since the fxn requires an arr but we can't declare one with
    // a non constant value...
    let mut arr = [0u8; 20];
    unsigned_int.write(&mut arr, &mut 0)?;
    Ok(arr[..unsigned_int.num_bytes()].to_vec())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use eos_chain::{AccountName, ActionName, PermissionLevel};

    use super::*;

    #[test]
    fn should_get_correct_action_digest_1() {
        let action = EosAction {
            account: AccountName::from_str("eosio.token").unwrap(),
            name: ActionName::from_str("transfer").unwrap(),
            authorization: vec![PermissionLevel::from_str("eosdididada3", "active").unwrap()],
            data: hex::decode("304c32c92597305510e874d820054dc6e87506000000000004454f5300000000083138333436393935")
                .unwrap(),
        };

        let result = hex::encode(get_action_digest(&action, true).unwrap());
        let expected_result = "9be5d1c3e18a4ae0c211f62f6885f6f17fd69e608191593663385e7a26301578";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_correct_action_digest_2() {
        let action = EosAction {
            name: ActionName::from_str("redeem").unwrap(),
            account: AccountName::from_str("btc.ptokens").unwrap(),
            authorization: vec![PermissionLevel::from_str("sbetbrescskr", "active").unwrap()],
            data: hex::decode("70214658dd93d5c140771b0000000000085042544300000022334e465047724c7a6f664571416a46446d34707967796e5444417939797436635765").unwrap(),
        };

        let result = hex::encode(get_action_digest(&action, true).unwrap());
        let expected_result = "80755c538467ddcc65190622a118316aa7ac1406bdfa8263c73e5a9653a8fd59";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_correct_action_digest_3() {
        let action = EosAction {
            name: ActionName::from_str("setabi").unwrap(),
            account: AccountName::from_str("eosio").unwrap(),
            authorization: vec![PermissionLevel::from_str("eosio.token", "active").unwrap()],
            data: hex::decode("00a6823403ea3055fd030e656f73696f3a3a6162692f312e310008076163636f756e7400010762616c616e636505617373657405636c6f73650002056f776e6572046e616d650673796d626f6c0673796d626f6c06637265617465000206697373756572046e616d650e6d6178696d756d5f737570706c790561737365740e63757272656e63795f7374617473000306737570706c790561737365740a6d61785f737570706c7905617373657406697373756572046e616d65056973737565000302746f046e616d65087175616e74697479056173736574046d656d6f06737472696e67046f70656e0003056f776e6572046e616d650673796d626f6c0673796d626f6c0972616d5f7061796572046e616d65067265746972650002087175616e74697479056173736574046d656d6f06737472696e67087472616e7366657200040466726f6d046e616d6502746f046e616d65087175616e74697479056173736574046d656d6f06737472696e6706000000000085694405636c6f73650000000000a86cd44506637265617465000000000000a531760569737375650000000000003055a5046f70656e0000000000a8ebb2ba0672657469726500000000572d3ccdcd087472616e736665720002000000384f4d1132036936340000076163636f756e740000000000904dc60369363400000e63757272656e63795f73746174730000000000").unwrap(),
        };

        let result = hex::encode(get_action_digest(&action, true).unwrap());
        let expected_result = "a5a9db34733bd34d033fd9ecbdb72712c56957e7948548464452fdf7ee27af5f";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_correct_action_digest_4() {
        let action = EosAction {
            account: AccountName::from_str("ptokensbtc1a").unwrap(),
            name: ActionName::from_str("redeem").unwrap(),
            authorization: vec![PermissionLevel::from_str("test1test2tt", "active").unwrap()],
            data: hex::decode("90b3c858e590b1ca50c3000000000000085042544300000023324e3238545a684c586468566546764e33706359464667744776686a37575574507737").unwrap(),
        };

        let result = hex::encode(get_action_digest(&action, false).unwrap());
        let expected_result = "364afa1cc13bca5dce1027f089e56889171373f66f5e3e59637251aaaeac4caa";
        assert_eq!(result, expected_result);
    }
}
