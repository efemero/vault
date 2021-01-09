use super::erc_20::Erc20Token;
use ethabi::{Address, Uint};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Vault {
    pub subscribed: bool,
    pub min_ratio: Uint,
    pub max_ratio: Uint,
    pub repay_ratio: Uint,
    pub boost_ratio: Uint,
    pub owner: Address,
    pub collateral: Uint,
    pub debt: Uint,
    pub token: Erc20Token,
}

#[derive(Debug, Clone)]
pub struct VaultInfo {
    pub col: Uint,
    pub debt: Uint,
}

impl Vault {
    pub fn get_dai_value(&self, price: Uint) -> Result<Uint, Box<dyn Error>> {
        let dai_col = price * self.collateral / Uint::exp10(self.token.decimals);
        Ok(dai_col - self.debt)
    }
    pub fn get_col_value(&self, price: Uint) -> Result<Uint, Box<dyn Error>> {
        let dai_value = self.get_dai_value(price)?;
        Ok(dai_value * Uint::exp10(self.token.decimals) / price)
    }

    pub fn get_vault_info(
        &self,
        _price: Uint,
        _low: Uint,
        _high: Uint,
        _friction: f64,
    ) -> Result<VaultInfo, Box<dyn Error>> {
        Ok(VaultInfo {
            col: self.collateral,
            debt: self.debt,
        })
    }
}
