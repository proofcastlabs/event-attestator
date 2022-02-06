use ethereum_types::U256;

use crate::types::Result;

pub trait DictionaryDecimalConverter {
    fn get_host_decimals(&self) -> Result<u16>;
    fn get_native_decimals(&self) -> Result<u16>;

    fn requires_decimal_conversion(&self) -> Result<bool> {
        Ok(self.get_host_decimals()? != self.get_native_decimals()?)
    }

    fn convert_native_amount_to_host_amount(&self, amount: U256) -> Result<U256> {
        self.convert_amount(amount, true)
    }

    fn convert_host_amount_to_native_amount(&self, amount: U256) -> Result<U256> {
        self.convert_amount(amount, false)
    }

    fn convert_amount(&self, amount: U256, convert_native_to_host: bool) -> Result<U256> {
        if self.requires_decimal_conversion()? {
            let host_decimals = self.get_host_decimals()?;
            let native_decimals = self.get_native_decimals()?;
            let to = if convert_native_to_host {
                host_decimals
            } else {
                native_decimals
            };
            let from = if convert_native_to_host {
                native_decimals
            } else {
                host_decimals
            };
            let multiplicand = U256::from(10).pow(U256::from(to));
            let divisor = U256::from(10).pow(U256::from(from));
            info!("✔ Converting {} from {} decimals to {}...", amount, from, to);
            Ok((amount * multiplicand) / divisor)
        } else {
            info!("✔ Amounts for this dictionary entry do NOT require decimal conversion!");
            Ok(amount)
        }
    }
}
