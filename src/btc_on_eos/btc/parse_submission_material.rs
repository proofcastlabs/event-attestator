use std::str::FromStr;
use bitcoin_hashes::sha256d;
use bitcoin::{
    consensus::encode::deserialize,
    blockdata::{
        block::Block as BtcBlock,
        block::BlockHeader as BtcBlockHeader,
        transaction::Transaction as BtcTransaction,
    },
};
use crate::btc_on_eos::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    btc::{
        btc_state::BtcState,
        btc_types::{
            BtcBlockJson,
            BtcBlockAndId,
            DepositInfoList,
            DepositAddressInfo,
            SubmissionMaterial,
            SubmissionMaterialJson,
            DepositAddressInfoJson,
            DepositAddressJsonList,
        },
    },
};

fn parse_btc_block_json_to_block_header(
    btc_block_json: BtcBlockJson
) -> Result<BtcBlockHeader> {
    trace!("✔ Parsing `BtcBlockJson` to `BtcBlockHeader`...");
    Ok(
        BtcBlockHeader::new(
            btc_block_json.timestamp,
            btc_block_json.bits,
            btc_block_json.nonce,
            btc_block_json.version,
            sha256d::Hash::from_str(&btc_block_json.merkle_root)?,
            sha256d::Hash::from_str(&btc_block_json.previousblockhash)?,
        )
    )
}

pub fn parse_btc_block_json_to_btc_block(
    json: &SubmissionMaterialJson
) -> Result<BtcBlock> {
    trace!("✔ Parsing `SubmissionMaterialJson` to `BtcBlock`...");
    Ok(
        BtcBlock::new(
            parse_btc_block_json_to_block_header(
                json.block.clone()
            )?,
            convert_hex_txs_to_btc_transactions(
                &json.transactions
            )?
        )
    )
}

pub fn parse_submission_material_to_json(
    submission_material: &str
) -> Result<SubmissionMaterialJson> {
    trace!("✔ Parsing JSON string to `SubmissionMaterialJson`...");
    match serde_json::from_str(submission_material) {
        Ok(json) => Ok(json),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }
}

fn convert_hex_tx_to_btc_transaction(hex: &String) -> Result<BtcTransaction> {
    Ok(deserialize::<BtcTransaction>(&hex::decode(hex)?)?)
}

fn convert_hex_txs_to_btc_transactions(
    hex_txs: &Vec<String>
) -> Result<Vec<BtcTransaction>> {
    hex_txs
        .iter()
        .map(convert_hex_tx_to_btc_transaction)
        .collect::<Result<Vec<BtcTransaction>>>()
}

fn parse_deposit_list_json_to_deposit_info(
    deposit_address_info_json: &DepositAddressInfoJson
) -> Result<DepositAddressInfo> {
    DepositAddressInfo::new(
        deposit_address_info_json.nonce,
        &deposit_address_info_json.address,
        &deposit_address_info_json.btc_deposit_address,
        &deposit_address_info_json.address_and_nonce_hash,
    )
}

fn parse_deposit_info_jsons_to_deposit_info_list(
    deposit_address_json_list: &DepositAddressJsonList
) -> Result<DepositInfoList> {
    deposit_address_json_list
        .iter()
        .map(parse_deposit_list_json_to_deposit_info)
        .collect::<Result<DepositInfoList>>()
}

pub fn parse_btc_block_from_submission_material(
    submision_material_json: &SubmissionMaterialJson,
) -> Result<BtcBlockAndId> {
    trace!("✔ Parsing `BtcBlockSAndtxsJson` to `BtcBlockAndId`...");
    Ok(
        BtcBlockAndId {
            height: submision_material_json.block.height,
            id: sha256d::Hash::from_str(&submision_material_json.block.id)?,
            deposit_address_list: parse_deposit_info_jsons_to_deposit_info_list(
                &submision_material_json.deposit_address_list,
            )?,
            block: parse_btc_block_json_to_btc_block(
                submision_material_json
            )?,
        }
    )
}

fn parse_submission_json(
    submission_json: &SubmissionMaterialJson,
) -> Result<SubmissionMaterial> {
    Ok(
        SubmissionMaterial {
            ref_block_num:
                submission_json.ref_block_num,
            ref_block_prefix:
                submission_json.ref_block_prefix,
            block_and_id:
                parse_btc_block_from_submission_material(submission_json)?,
        }
    )
}

pub fn parse_submission_material_and_put_in_state<D>(
    submission_json: String,
    state: BtcState<D>,
) -> Result<BtcState<D>>
   where D: DatabaseInterface
{
    info!("✔ Parsing BTC submisson material...");
    parse_submission_material_to_json(&submission_json)
        .and_then(|json| parse_submission_json(&json))
        .and_then(|result| state.add_btc_submission_material(result))
}

#[cfg(test)]
mod tests {
    use super::*;
/* TODO reinstate once we have test vectors!
    #[test]
    fn should_parse_btc_block_json() {
        let string = get_sample_btc_block_json_string();
        if let Err(e) = parse_submission_material_to_json(&string) {
            panic!("Error getting json from btc block and txs sample: {}", e);
        }
    }

    #[test]
    fn should_parse_block_and_tx_json_to_struct() {
        let json = get_sample_btc_block_json()
            .unwrap();
        if let Err(e) = parse_btc_block_from_submission_material(json) {
            panic!("Error getting json from btc block and txs sample: {}", e);
        }
    }

    #[test]
    fn should_not_panic_deserializing_tx() {
        let tx_bytes = hex::decode("0200000000010117c33a062c8d0c2ce104c9988599f6ba382ff9f786ad48519425e39af23da9880000000000feffffff022c920b00000000001976a914be8a09363cd4719b1c05b2703797ca890b718b5088acf980d30d000000001600147448bbdfe47ec14f27c68393e766567ac7c9c77102473044022073fc2b43d5c5f56d7bc92b47a28db989e04988411721db96fb0eea6689fb83ab022034b7ce2729e867962891fec894210d0faf538b971d3ae9059ebb34358209ec9e012102a51b8eb0eb8ef6b2a421fb1aae3d7308e6cdae165b90f78074c2493af98e3612c43b0900")
            .unwrap();
        if let Err(e) = deserialize::<BtcTransaction>(&tx_bytes) {
            panic!("Error deserializing tx: {}", e);
        }
    }
    */

    #[test]
    fn should_parse_deposit_list_json_to_deposit_info() {
        let nonce = 1578079722;
        let address = "0xedb86cd455ef3ca43f0e227e00469c3bdfa40628"
            .to_string();
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ"
            .to_string();
        let address_and_nonce_hash =
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae"
            .to_string();
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            btc_deposit_address,
            address_and_nonce_hash,
        };
        if let Err(e) = parse_deposit_list_json_to_deposit_info(&deposit_json) {
            panic!("Error parsing deposit info json: {}", e);
        }
    }
}
