#![allow(dead_code)] //  TODO FIXME: Rm!
use bitcoin_hashes::{
    Hash,
    sha256,
};
use crate::btc_on_eos::{
    errors::AppError,
    types::{
        Byte,
        Bytes,
        Result,
    },
};

pub type CanonicalLeft = Bytes;
pub type CanonicalRight = Bytes;
pub type Sha256Hash = bitcoin_hashes::sha256::Hash;
pub type CanonicalPair = (CanonicalLeft, CanonicalRight);

fn set_first_bit_of_byte_to_zero(mut byte: Byte) -> Byte { // Left
    byte &= 0b0111_1111;
    byte
}

fn set_first_bit_of_byte_to_one(mut byte: Byte) -> Byte { // Right
    byte |= 0b1000_0000;
    byte
}

fn set_first_bit_of_hash_to_one(mut hash: Bytes) -> Bytes {
    hash[0] = set_first_bit_of_byte_to_one(hash[0]);
    hash
}

fn set_first_bit_of_hash_to_zero(mut hash: Bytes) -> Bytes {
    hash[0] = set_first_bit_of_byte_to_zero(hash[0]);
    hash
}

fn make_canonical_left(hash: Bytes) -> CanonicalLeft {
    set_first_bit_of_hash_to_zero(hash)
}

fn make_canonical_right(hash: Bytes) -> CanonicalRight {
    set_first_bit_of_hash_to_one(hash)
}

fn is_canonical_left(hash: &Bytes) -> bool {
    hash[0] & 0b1000_0000 == 0
}

fn is_canonical_right(hash: &Bytes) -> bool {
    hash[0] & 0b1000_0000 == 0b1000_0000
}

fn make_canonical_pair(l: Bytes, r: Bytes) -> CanonicalPair {
    (
        make_canonical_left(l),
        make_canonical_right(r),
    )
}

fn concatenate_canonical_pair(mut pair: CanonicalPair) -> Bytes {
    pair.0.append(& mut pair.1);
    pair.0
}

fn hash_canonical_pair(pair: CanonicalPair) -> Sha256Hash {
    sha256::Hash::hash(&concatenate_canonical_pair(pair))
}

fn make_and_hash_canonical_pair(l: Bytes, r: Bytes) -> Bytes {
    hash_canonical_pair(make_canonical_pair(l, r)).to_vec()
}

fn get_merkle_digest(mut digests: Vec<Bytes>) -> Bytes {
    if digests.len() == 0 {
        return vec![0x00] // TODO Need a type for this!
    }
    while digests.len() > 1 {
        if digests.len() % 2 != 0 {
            println!("is odd!");
            let last = digests[digests.len() - 1].clone();
            digests.push(last);
        }
        for i in 0..(digests.len() / 2) {
            println!(
                "Loop #{}. Making pair from:\n{}\n{}",
                i,
                hex::encode(digests[2 * i].clone()),
                hex::encode(digests[(2 * i) + 1].clone()),
            );
            let canon_pair = make_canonical_pair(
                digests[2 * i].clone(),
                digests[(2 * i) + 1].clone(),
            );
            println!(
                "Canonical pair: \n{}\n{}",
                hex::encode(canon_pair.0.clone()),
                hex::encode(canon_pair.1.clone()),
            );
            let hash = hash_canonical_pair(canon_pair);
            println!("Hashed:\n{}\n", hex::encode(hash));

            digests[i] = hash_canonical_pair(
                make_canonical_pair(
                    digests[2 * i].clone(),
                    digests[(2 * i) + 1].clone(),
                )
            ).to_vec();
        }
        digests.resize(digests.len() / 2, vec![0x00]);
    }
    digests[0].clone()
}

fn generate_merkle_proof(mut index: usize, mut ids: Vec<Bytes>) -> Result<Vec<Bytes>> { // TODO Type?
    // Note this works for the guys tests on the PR on EOSIO
    // https://github.com/EOSIO/eos/pull/7847/files
    let mut proof = Vec::new();
    proof.push(ids[index].clone());
    match index < ids.len() {
        false => Err(AppError::Custom("✘ Index out of bounds!".to_string())),
        true => {
            while ids.len() > 1 {
                if ids.len() % 2 != 0 {
                    let last = ids[ids.len() - 1].clone();
                    ids.push(last) // Can use vector last to get an Optional value?
                }

                for i in 0..ids.len() / 2 {
                    if index / 2 == i {
                        if index % 2 != 0 {
                            println!(
                                "Pushing into proof: {}",
                                hex::encode(make_canonical_left(ids[2 * i].clone()))
                            );
                            proof.push(make_canonical_left(ids[2 * i].clone()))
                        } else {
                            println!(
                                "Pushing into proof: {}",
                                hex::encode(make_canonical_right(ids[2 * i + 1].clone()))
                            );
                            proof.push(make_canonical_right(ids[2 * i + 1].clone()))
                        }
                        index /= 2; // Why is this here? (Because loops stops based on vec size!)
                    }
                    ids[i] = hash_canonical_pair(
                        make_canonical_pair(ids[2 * i].clone(), ids[2 * i + 1].clone())
                    ).to_vec()
                }
                ids.resize(ids.len() / 2, vec![0x00]);
            }
            println!("Pushing into proof: {}", hex::encode(ids[0].clone()));
            proof.push(ids[0].clone());
            return Ok(proof);
        }
    }
}

fn verify_merkle_proof(proof: Vec<Bytes>) -> bool {
    // This seems to check the final vs the first but even then doesn't work?!
    let mut node = proof[0].clone();
    for i in 1..proof.len() { // proof.len() - 1
        if is_canonical_right(&proof[i]) {
            println!(" Left hash: {}", hex::encode(node.clone()));
            println!("Right hash: {}", hex::encode(proof[i].clone()));
            node = hash_canonical_pair((
                node,
                proof[i].clone()
            )).to_vec();
            println!("  The node: {}", hex::encode(node.clone()));
        } else {
            println!(" Left hash: {}", hex::encode(proof[i].clone()));
            println!("Right hash: {}", hex::encode(node.clone()));
            node = hash_canonical_pair((
                proof[i].clone(),
                node
            )).to_vec();
            println!("  The node: {}", hex::encode(node.clone()));
        }
    }
    println!("The final output: {}", hex::encode(node.clone()));
    println!("  The exp output: {}", hex::encode(proof.last().unwrap()));
    Some(&node) == proof.last()
}

#[cfg(test)]
mod tests {
    use hex;
    use super::*;
    use std::str::FromStr;
    use eos_primitives::{
        Action,
        ActionName,
        AccountName,
        AuthSequence,
        ActionReceipt,
        SerializeData,
        PermissionName,
        PermissionLevel,
    };

    fn get_expected_digest_1() -> &'static str {
        "9b9babebfbdff48ce4002b5f3c7f999c0ee74707b6d121c47ef5db68c6be7262"
    }

    fn get_expected_digest_2() -> &'static str {
        "122cd09d66ca7df007a35bd9c9be5484833f1a69ad0c8527c3e2a56b6955e761"
    }

    fn get_expected_digest_bytes_1() -> Bytes {
        hex::decode(get_expected_digest_1()).unwrap()
    }

    fn get_expected_digest_bytes_2() -> Bytes {
        hex::decode(get_expected_digest_2()).unwrap()
    }

    fn get_expected_first_byte_1() -> Byte {
        0b0001_1011
    }

    fn get_expected_first_byte_2() -> Byte {
        0b1001_0010
    }

    fn get_sample_canonical_pair() -> CanonicalPair {
        make_canonical_pair(
            get_expected_digest_bytes_1(),
            get_expected_digest_bytes_2(),
        )
    }

    #[test]
    fn should_set_first_bit_of_byte_to_zero() {
        let byte = 0b1011_1011;
        let expected_result = 0b0011_1011;
        let result = set_first_bit_of_byte_to_zero(byte);
        assert!(result == expected_result);
    }

    #[test]
    fn should_set_first_bit_of_byte_to_one() {
        let byte = 0b0011_0011;
        let expected_result = 0b1011_0011;
        let result = set_first_bit_of_byte_to_one(byte);
        assert!(result == expected_result);
    }

    #[test]
    fn should_set_first_bit_of_hash_to_one() {
        let hash = get_expected_digest_bytes_2();
        let result = set_first_bit_of_hash_to_one(hash.clone());
        for i in 0..hash.len() {
            if i == 0 {
                assert!(result[i] == get_expected_first_byte_2());
            } else {
                assert!(result[i] == hash[i]);
            }
        }
    }

    #[test]
    fn should_set_first_bit_of_hash_to_zero() {
        let hash = get_expected_digest_bytes_1();
        let result = set_first_bit_of_hash_to_zero(hash.clone());
        for i in 0..hash.len() {
            if i == 0 {
                assert!(result[i] == get_expected_first_byte_1());
            } else {
                assert!(result[i] == hash[i]);
            }
        }
    }

    #[test]
    fn should_make_hash_canonical_right() {
        let hash = get_expected_digest_bytes_2();
        let result = make_canonical_right(hash.clone());
        for i in 0..hash.len() {
            if i == 0 {
                assert!(result[i] == get_expected_first_byte_2());
            } else {
                assert!(result[i] == hash[i]);
            }
        }
    }

    #[test]
    fn should_make_hash_canonical_left() {
        let hash = get_expected_digest_bytes_1();
        let result = make_canonical_left(hash.clone());
        for i in 0..hash.len() {
            if i == 0 {
                assert!(result[i] == get_expected_first_byte_1());
            } else {
                assert!(result[i] == hash[i]);
            }
        }
    }

    #[test]
    fn canonical_left_hash_should_be_canonical_left() {
        let hash = get_expected_digest_bytes_1();
        let canonical_left_hash = make_canonical_left(hash.clone());
        let is_left = is_canonical_left(&canonical_left_hash);
        let is_right = is_canonical_right(&canonical_left_hash);
        assert!(is_left);
        assert!(!is_right);
    }

    #[test]
    fn canonical_right_hash_should_be_canonical_right() {
        let hash = get_expected_digest_bytes_2();
        let canonical_right_hash = make_canonical_right(hash.clone());
        let is_left = is_canonical_left(&canonical_right_hash);
        let is_right = is_canonical_right(&canonical_right_hash);
        assert!(!is_left);
        assert!(is_right);
    }

    #[test]
    fn should_get_correct_action_digest() {
        let account_name = AccountName::from_str("provabletokn").unwrap();
        let action_name = ActionName::from_str("event").unwrap();
        let actor = AccountName::from_str("provabletokn").unwrap();
        let permission = PermissionName::from_str("active").unwrap();
        let permission_level = PermissionLevel { actor, permission };
        let authorization = vec![permission_level];
        let data = hex::decode(
            "e0d2b86b1a3962343021cd2a1eb3e9ad672b00000000000004454f53000000002a3078303236644336413433353631444138413641373735353338623139324133653933366330463239422301000000000000"
            ).unwrap();
        let action = Action::new(
            account_name,
            action_name,
            authorization,
            data,
        );
        let serialized_action = action.to_serialize_data();
        let result = sha256::Hash::hash(&serialized_action).to_string();
        assert!(result == get_expected_digest_1().to_string());
    }

    #[test]
    fn should_make_canonical_pair() {
        let digest_1 = get_expected_digest_bytes_1();
        let digest_2 = get_expected_digest_bytes_2();
        let result = make_canonical_pair(digest_1.clone(), digest_2.clone());
        for i in 0..result.0.len() {
            if i == 0 {
                assert!(result.0[i] == get_expected_first_byte_1());
            } else {
                assert!(result.0[i] == digest_1[i]);
            }
        }
        for i in 0..result.1.len() {
            if i == 0 {
                assert!(result.1[i] == get_expected_first_byte_2());
            } else {
                assert!(result.1[i] == digest_2[i]);
            }
        }
    }

    #[test]
    fn should_hash_canonical_pair() {
        let expected_result =
            "a26284468e89fe4a5cce763ca3b3d3d37d5fcb35f289c63f0558487ec57ace28";
        let canonical_pair = get_sample_canonical_pair();
        let result = hash_canonical_pair(canonical_pair);
        println!("{}", result);
        assert!(result.to_string() == expected_result);
    }

    #[test]
    fn should_serialize_a_simple_action_receipt_correctly() {
        let expected_result =
            "6cd473b189a292bd520cac3430cc7934273da81cc3417376194a5d757b4abdc8"
                .to_string();
        let result = ActionReceipt::new(
            "eosio",
            "a6a370c6569034a4cc41935dd88f83d1c64e0414580872f29d87f69fe7a5d769",
            60725518,
            12,
            498637597,
            10,
            vec![AuthSequence::new("eosio", 59191700).unwrap()],
        )
            .unwrap()
            .to_digest();
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_merkle_root_for_an_even_number_of_action_receipts() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58316764
        let expected_result =
            "2f013d3ed57c89f1824772d18a4a74c043574bad47e9c6f088136e7595511810";
        let action_digest_1 = ActionReceipt::new(
            "eosio",
            "8e3e721a497dbae5e5fde0bb43e9086628809efaf102b763a3e9820adce9ce8f",
            62815613,
            12,
            503056735,
            10,
            vec![AuthSequence::new("eosio", 61275209).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_2 = ActionReceipt::new(
            "provabletokn",
            "4b991cebb3e6667b242aca3fb011623cd8ce2be2e8c24958da551c7b3ba68903",
            2884,
            48,
            503056736,
            80,
            vec![AuthSequence::new("provabletokn", 3090).unwrap()],
        )
            .unwrap()
            .to_digest();
        let result = get_merkle_digest(vec![ action_digest_1, action_digest_2]);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_merkle_root_for_an_odd_number_of_action_receipts_gt_one() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58319528
        let expected_result =
            "593f54cbc0b877b30cec5e510838b2b16ca00aca43e21d204d21eb8e8f947aa0";
        let action_digest_1 = ActionReceipt::new(
            "eosio",
            "23ab74b930cceea6061e1c4580ec988bf483a77e225cfca254d832928b4d1b36",
            62818486,
            12,
            503062766,
            10,
            vec![AuthSequence::new("eosio", 61277973).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_2 = ActionReceipt::new(
            "eosebetbullj",
             "b9243d8513e25705e89d7ccd0491f4a57d07b9866fd89d3446887af852cfed15",
             1440226,
             215,
             503062767,
             215,
            vec![AuthSequence::new("eosebetbullj", 1440687).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_3 = ActionReceipt::new(
             "dvmh1tbb1him",
             "4bd1d3e987cd13e3d108a9a0cd185bf022cb1a826f69f163fcd109db54ba799f",
             804629,
             2,
             503062768,
             1,
            vec![AuthSequence::new("dvmh1tbb1him", 804649).unwrap()],
        )
            .unwrap()
            .to_digest();
        let result = get_merkle_digest(vec![
            action_digest_1,
            action_digest_2,
            action_digest_3,
        ]);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_action_mroot_when_action_has_gt_one_auth_sequence() {
        // NOTE: https://jungle.bloks.io/block/58345436
        let expected_result =
            "f93a91688d12170c24807d4bd507cf52dcde962ae4a41a86fe55231dee4df348";
        let action_receipt_1 = ActionReceipt::new(
            "eosio",
            "2d5371b958af052629f3fb62ede1bfcd94703675bc734535bf87fb615284dba3",
            62844592,
            12,
            503124645,
            10,
            vec![AuthSequence::new(
                "eosio",
                61303891
            ).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_receipt_2 = ActionReceipt::new(
            "authsequence",
            "ae341469a7b3936c70e9684a42ef8fc1975f1bb2fe1f3b0b1105eda7d3a6276a",
            10,
            1,
            503124646,
            1,
            vec![
                AuthSequence::new("othrsequence", 14).unwrap(),
                AuthSequence::new("rick11111111", 268).unwrap()
            ]
        )
            .unwrap()
            .to_digest();
        let result = get_merkle_digest(vec![
            action_receipt_1,
            action_receipt_2,
        ]);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_action_mroot_for_four_actions_correctly() {
        let expected_result =
            "8b4e5e5d3e7587065896d0076d65c72e03c11a9159d414eb3a2363b59108116a";
        let action_digest_1 = ActionReceipt::new(
            "eosio",
            "3b434aa9331f5e2a0e7a0060d576fa6688406667100bdf3458104dede44ec4e9",
            62826453,
            12,
            503081363,
            10,
            vec![AuthSequence::new(
                "eosio",
                61285932
            ).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_2 = ActionReceipt::new(
            "pokerpokerts",
            "3d380413463e8716ef9c1f8c853dfab0c70f209cce75cae9a5b74e4e678a68a0",
            241512,
            4,
            503081364,
            30,
            vec![AuthSequence::new(
                "pokerpokerts",
                241552
            ).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_3 = ActionReceipt::new(
            "oracleoracle",
            "065527f0429dfa9bb79575ec5270b20f714fb9e61a9ce6ba9c86b2e69a773f82",
            531231,
            2,
            503081365,
            2,
            vec![AuthSequence::new(
                "feeder111112",
                152730
            ).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_4 = ActionReceipt::new(
            "dvmh1tbb1him",
            "18e42aa86473509cf620764ca606136b037e1a8ee6fb8efaa8fa657c7fa2fffc",
            805647,
            2,
            503081366,
            1,
            vec![AuthSequence::new(
                "dvmh1tbb1him",
                805667
            ).unwrap()],
        )
            .unwrap()
            .to_digest();
        let result = get_merkle_digest(vec![
            action_digest_1,
            action_digest_2,
            action_digest_3,
            action_digest_4,
        ]);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_action_mroot_for_four_actions_correctly_2() {
        // NOTE https://jungle.bloks.io/block/73202214
        let expected_action_mroot =
            "33c4712389ebdb79d4cc726c15924870f2cd61ac64078965feb03af85121df44";
        let action_receipts = vec![
            ActionReceipt::new(
                "eosio",
                "d6a9ba740b3f1911113c9a68629b2d98052021a9ab1d6966542803e32a6ac610",
                78206122,
                14,
                548441354,
                13,
                vec![
                    AuthSequence::new(
                        "eosio",
                        76220906
                    ).unwrap(),
                ],
            ).unwrap().to_digest(),
            ActionReceipt::new(
                "bigbangzhiya",
                "7549e59646a1a0dac4b294c5d8f672ea4dc9a93542a8d2a3bbd605251ac40bf6",
                1090423,
                49,
                548441355,
                327,
                vec![
                    AuthSequence::new(
                        "bigbangweija",
                        1086639
                    ).unwrap(),
                ],
            ).unwrap().to_digest(),
            ActionReceipt::new(
                "eosio.msig",
                "49d7375f322dcbfdc50750a53bddc60b4c119b3bee031112150e477c7c75e201",
                131343,
                4,
                548441356,
                6,
                vec![
                    AuthSequence::new(
                        "bbaseprod222",
                        27753
                    ).unwrap(),
                ],
            ).unwrap().to_digest(),
            ActionReceipt::new(
                "hotchaintest",
                "e1c1c26550f233beb4f26303912afdb59cdb926969d06f7ecb66b770673be07c",
                423147,
                1,
                548441357,
                1,
                vec![
                    AuthSequence::new(
                        "hotchaintest",
                        423149
                    ).unwrap(),
                ],
            ).unwrap().to_digest(),
        ];
        let result = get_merkle_digest(action_receipts);
        assert_eq!(hex::encode(result), expected_action_mroot);
    }

    // NOTE NEED A 5
    #[test]
    fn should_get_action_mroot_for_six_actions_correctly() {
        // NOTE https://jungle.bloks.io/block/73202274
        let expected_action_mroot =
            "a370588d26b75b0a556caca8e0fb9ddedd8edeb9965c4c9d09280b382d93b866";
        let action_receipts = vec![
            ActionReceipt::new(
                "eosio",
                "5749896467bb318cb017b5713e297ce52a8d1d49642dbacc5751dcf75ca8dbf5",
                78206182,
                14,
                548441442,
                13,
                vec![
                    AuthSequence::new(
                        "eosio",
                        76220966
                    ).unwrap()
                ],
            ).unwrap().to_digest(),
            ActionReceipt::new(
                "eosio.msig",
                "c148f2a0ff794af6205316d246f4f36330f6319e909732cc44de799a192f7e9f",
                131350,
                4,
                548441443,
                6,
                vec![
                    AuthSequence::new(
                        "bbaseprod444",
                        13564
                    ).unwrap()
                ],
            ).unwrap().to_digest(),
            ActionReceipt::new(
                "eosio.msig",
                "0be0c5a09e156ae1b6be7a39d8ee70d734e5b1b558b9b758a5a43e6343f9e7e7",
                131351,
                4,
                548441444,
                6,
                vec![
                    AuthSequence::new(
                        "bbaseprod333",
                        13350
                    ).unwrap()
                ],
            ).unwrap().to_digest(),
            ActionReceipt::new(
                "hotchaintest",
                "edfbb256413c0d95a6146f694afe5dd399ccf012b0b0231b30c50ecb1bcd9b65",
                423155,
                1,
                548441445,
                1,
                vec![
                    AuthSequence::new(
                        "hotchaintest",
                        423157
                    ).unwrap()
                ],
            ).unwrap().to_digest(),
            ActionReceipt::new(
                "bigbangzhiya",
                "3a6f7b0c7ce45a1e731c36a36a5e77834530f67cfbcf7f86f7ebfdf3160a0818",
                1090426,
                49,
                548441446,
                327,
                vec![
                    AuthSequence::new(
                        "bigbangweija",
                        1086642
                    ).unwrap()
                ]
            ).unwrap().to_digest(),
            ActionReceipt::new(
                "endlessgame1",
                "0c44294a4cf2689e9ed6252418f1eb63467eab3cb4afc2ea76d33cf8733e4304",
                103759,
                62,
                548441447,
                62,
                vec![
                    AuthSequence::new(
                        "wesleytesta2",
                        49572
                    ).unwrap()
                ],
            ).unwrap().to_digest(),
        ];
        let result = get_merkle_digest(action_receipts);
        assert_eq!(hex::encode(result), expected_action_mroot);
    }

    /* TODO Re-instate!
    #[test]
    fn should_get_action_mroot_for_seven_actions_correctly() {
        // FIXME: Doesn't work!
        // NOTE https://jungle.bloks.io/block/73202313
        let expected_action_mroot =
            "f0237acdf5d07696de26e5fb6f5695c112b2bf458d70dc753de10339101c5c8d";
        let mut action_receipts = vec![
            ActionReceipt::new( //#0
                "eosio",
                "de4e9f8c2488dbbc0821ea21cebcdbafcaae1f685915379f6c01e83edc8b7737",
                78206221,
                14,
                548441513,
                13,
                vec![
                    AuthSequence::new(
                        "eosio",
                        76221005
                    ).unwrap(),
                ],
            ).unwrap(),
            ActionReceipt::new( //#1
                "liquidxcnsmr",
                "8ae54025edebbc244ad870a54150758edd7dc12f34c3f516034ed2a93ff8d23c",
                4188,
                38,
                548441514,
                41,
                vec![
                    AuthSequence::new(
                        "heliosselene",
                        2960
                    ).unwrap(),
                ],
            ).unwrap(),
            ActionReceipt::new( //#2
                "dappservicex",
                "9ce1f8804925e8e9428f3d143330eae87d5c74d6ce65696be0a82208d0fffdcb",
                3791,
                1,
                548441517,
                4,
                vec![
                    AuthSequence::new(
                        "heliosselene",
                        2961
                    ).unwrap(),
                ],
            ).unwrap(),
            ActionReceipt::new( // #3
                "liquidxcnsmr",
                "bd72bce18912267b13899a36aeeb2f2750fbc9a98a69f3389831f7f7ff4451f9",
                4189,
                38,
                548441515,
                41,
                vec![
                    AuthSequence::new(
                        "liquidxcnsmr",
                        4350
                    ).unwrap(),
                ],
            ).unwrap(),
            ActionReceipt::new( // #4
                "dappservices",
                "bd72bce18912267b13899a36aeeb2f2750fbc9a98a69f3389831f7f7ff4451f9",
                419,
                38,
                548441516,
                41,
                vec![
                    AuthSequence::new(
                        "liquidxcnsmr",
                        4351
                    ).unwrap(),
                ],
            ).unwrap(),
            ActionReceipt::new( //#5
                "oracleoracl2",
                "7896ed2fcc2c17d9eecf8be903b805e04f4571c612b2ceaed4b0c5f82e01973e",
                247913,
                1,
                548441518,
                2,
                vec![
                    AuthSequence::new(
                        "feeder111111",
                        672806
                    ).unwrap(),
                ],
            ).unwrap(),
            ActionReceipt::new( // #6
                "hotchaintest",
                "55fed4e14a9d5990a35db3f0d7b8146ae9179145752ef627aa7ae6e43fd7aa4b",
                423160,
                1,
                548441519,
                1,
                vec![
                    AuthSequence::new(
                        "hotchaintest",
                        423162
                    ).unwrap(),
                ],
            ).unwrap(),
        ];
        action_receipts
            .sort_by(|a, b| b.global_sequence.cmp(&a.global_sequence));
        let action_digests = action_receipts
            .iter()
            .map(|receipt| receipt.to_digest())
            .collect::<Vec<Bytes>>();
        let digests_hex = action_digests
            .iter()
            .map(|digest| hex::encode(digest))
            .collect::<Vec<String>>();
        let expected_digests_hex = vec![ // NOTE: Derived from EOS source code
            "23e782cf962eb3c61891a0ef39c6302c950d547ee5f8e67dc8215ab28b730649",
            "cd4060385f2efc2dbc29bcd9f294101751d470794b4f9a899f0ae8d093534fc1",
            "b3cc210b235c1ed3b1ab0460d9e07dd39d182b69a0735181ed70186e1e70698b",
            "20b62a4fda167c0882ccdbb95ecd776768558d7fba9ce1fb5c51b3705ab1a457",
            "594854e55088e859190c57a5cc67055fc01878facebb7415eec364687578a21e",
            "bf865aaafe362a3256322ac7b1e6090dac1cb01b3ed711f57f5856ced3718c24",
            "5350e430c555bd6cfefbebdc0239fb6a8217237cc8d6f0fb8c53dbf089e77788",
        ];
        digests_hex
            .iter()
            .enumerate()
            .map(|(i, digest)| assert_eq!(digest, expected_digests_hex[i]))
            .for_each(drop);
        let result = get_merkle_digest(action_digests);
        assert_eq!(hex::encode(result), expected_action_mroot);
    }

    #[test]
    fn should_get_action_mroot_for_nine_actions_correctly() {
        let action_receipts = vec![
            ActionReceipt::new(
                "eosio",
                "6830388759a058b8264ce57d52586472aa097817d6650e844ba3b7b2dc871412",
                78199371,
                14,
                548430849,
                13,
                vec![AuthSequence::new("eosio",76214151).unwrap()],
            )
                .unwrap()
                .to_digest(),
            ActionReceipt::new(
                "liquidxcnsmr",
                "20d6f74f4e2bc86e1ccaa0bb7d074238194b20e4567b5d4c1a96352434b37e29",
                4070,
                38,
                548430850,
                41,
                vec![AuthSequence::new("heliosselene",2730).unwrap()],
            )
                .unwrap()
                .to_digest(),
            ActionReceipt::new(
                "dappservicex",
                "9ce1f8804925e8e9428f3d143330eae87d5c74d6ce65696be0a82208d0fffdcb",
                3676,
                1,
                548430853,
                4,
                vec![AuthSequence::new("heliosselene",2731).unwrap()],
            )
                .unwrap()
                .to_digest(),
            ActionReceipt::new(
                "liquidxcnsmr",
                "9855ac83c35a99ade3384612f9a67ac719d529e59959643c5d7e789bb8bee3ac",
                4071,
                38,
                548430851,
                41,
                vec![AuthSequence::new("liquidxcnsmr",4232).unwrap()],
            )
                .unwrap()
                .to_digest(),
            ActionReceipt::new(
                 "dappservices",
                "9855ac83c35a99ade3384612f9a67ac719d529e59959643c5d7e789bb8bee3ac",
                304,
                38,
                548430852,
                41,
                vec![AuthSequence::new("liquidxcnsmr",4233).unwrap()],
            )
                .unwrap()
                .to_digest(),
            ActionReceipt::new(
                "eosio.msig",
                "6556130f9a188425fc35fbc541121b3890604afad0de82acbaba100475a21f28",
                130945,
                4,
                548430854,
                6,
                vec![AuthSequence::new("bbaseprod111",27699).unwrap()],
            )
                .unwrap()
                .to_digest(),
            ActionReceipt::new(
                "eosio.token",
                "2b173e132cbfa9d838abd52e004ca0414641989fb711c3449f95c44b690278c2",
                79389769,
                4,
                548430855,
                5,
                vec![AuthSequence::new("provabletest",4246).unwrap()],
            )
                .unwrap()
                .to_digest(),
            ActionReceipt::new(
                "provabletest",
                "2b173e132cbfa9d838abd52e004ca0414641989fb711c3449f95c44b690278c2",
                2033,
                4,
                548430856,
                5,
                vec![AuthSequence::new("provabletest",4247).unwrap()],
            )
                .unwrap()
                .to_digest(),
            ActionReceipt::new(
                "provabletokn",
                "2b173e132cbfa9d838abd52e004ca0414641989fb711c3449f95c44b690278c2",
                4375,
                4,
                548430857,
                5,
                vec![AuthSequence::new("provabletest",4248).unwrap()],
            )
                .unwrap()
                .to_digest(),
        ];
        println!("action receipts {:?}", action_receipts);
        let result = get_merkle_digest(action_receipts);
        println!("{}", hex::encode(result))
    }
*/

}
