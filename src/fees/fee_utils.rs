use crate::utils::convert_unix_timestamp_to_human_readable;

pub fn get_last_withdrawal_date_as_human_readable_string(timestamp: u64) -> String {
    if timestamp == 0 {
        "Fees have not yet been withdrawn!".to_string()
    } else {
        convert_unix_timestamp_to_human_readable(timestamp)
    }
}
