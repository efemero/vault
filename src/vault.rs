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

    pub fn get_up_price(&self) -> Result<Uint, Box<dyn Error>> {
        let up_price = self.debt * self.max_ratio / self.collateral;
        Ok(up_price)
    }

    pub fn get_down_price(&self) -> Result<Uint, Box<dyn Error>> {
        let down_price = self.debt * self.min_ratio / self.collateral;
        Ok(down_price)
    }

    pub fn get_up_dai_to_draw(&self) -> Result<Uint, Box<dyn Error>> {
        let up_price = self.get_up_price()?;
        let dai_value = self.get_dai_value(up_price)?;
        let final_debt = dai_value * Uint::exp10(18) / (self.boost_ratio - Uint::exp10(18));
        let dai_to_draw = final_debt - self.debt;
        Ok(dai_to_draw)
    }

    pub fn get_down_dai_to_payback(&self) -> Result<Uint, Box<dyn Error>> {
        let down_price = self.get_down_price()?;
        let dai_value = self.get_dai_value(down_price)?;
        let final_debt = dai_value * Uint::exp10(18) / (self.repay_ratio - Uint::exp10(18));
        let dai_to_payback = self.debt - final_debt;
        Ok(dai_to_payback)
    }

    pub fn get_up_vault(&self) -> Result<Vault, Box<dyn Error>> {
        let mut up_vault = self.clone();
        let dai_to_draw = up_vault.get_up_dai_to_draw()?;
        let up_price = up_vault.get_up_price()?;
        up_vault.collateral = up_vault.collateral + (dai_to_draw * Uint::exp10(18) / up_price);
        up_vault.debt = up_vault.debt + dai_to_draw;
        Ok(up_vault)
    }

    pub fn get_down_vault(&self) -> Result<Vault, Box<dyn Error>> {
        let mut down_vault = self.clone();
        let dai_to_payback = down_vault.get_down_dai_to_payback()?;
        let down_price = down_vault.get_down_price()?;
        down_vault.collateral =
            down_vault.collateral - (dai_to_payback * Uint::exp10(18) / down_price);
        down_vault.debt = down_vault.debt - dai_to_payback;
        Ok(down_vault)
    }

    pub fn predict_vault(&self, price: Uint, friction: f64) -> Result<Vault, Box<dyn Error>> {
        let up_price = self.get_up_price()?;
        let down_price = self.get_down_price()?;
        let mut vault = self.clone();
        if price > up_price {
            vault = vault.get_up_vault()?.predict_vault(price, friction)?;
        }
        if price < down_price {
            vault = vault.get_down_vault()?.predict_vault(price, friction)?;
        }
        vault.debt = Uint::from((vault.debt.as_u128() as f64 * (1.0 - friction)) as u128);
        vault.collateral =
            Uint::from((vault.collateral.as_u128() as f64 * (1.0 - friction)) as u128);
        Ok(vault)
    }

    pub fn show(&self, price: Uint, btc_price: f64) -> Result<(), Box<dyn Error>> {
        let price_f64 = price.as_u128() as f64 / Uint::exp10(18).as_u128() as f64;
        let dai_value =
            self.get_dai_value(price)?.as_u128() as f64 / Uint::exp10(18).as_u128() as f64;
        let eur_value = dai_value / 1.2271;
        let col_value =
            self.get_col_value(price)?.as_u128() as f64 / Uint::exp10(18).as_u128() as f64;
        let btc_value = col_value * btc_price;
        let down_price = self.get_down_price()?.as_u128() as f64 / Uint::exp10(18).as_u128() as f64;
        let up_price = self.get_up_price()?.as_u128() as f64 / Uint::exp10(18).as_u128() as f64;
        let max_ratio_pc = self.max_ratio.as_u128() as f64 / Uint::exp10(16).as_u128() as f64;
        let min_ratio_pc = self.min_ratio.as_u128() as f64 / Uint::exp10(16).as_u128() as f64;
        let boost_ratio_pc = self.boost_ratio.as_u128() as f64 / Uint::exp10(16).as_u128() as f64;
        let repay_ratio_pc = self.repay_ratio.as_u128() as f64 / Uint::exp10(16).as_u128() as f64;

        let col_dai =
            self.collateral.as_u128() as f64 * price_f64 / Uint::exp10(18).as_u128() as f64;
        let debt = self.debt.as_u128() as f64 / Uint::exp10(18).as_u128() as f64;
        let ratio_pc = col_dai * 100.0 / debt;
        println!("{:<11}: {:>9.2} ({:.2}%)", "price", price_f64, ratio_pc);
        println!(
            "{:<11}: {:>9.2} ({}% -> {}%)",
            "down price", down_price, min_ratio_pc, repay_ratio_pc
        );
        println!(
            "{:<11}: {:>9.2} ({}% -> {}%) ",
            "up price", up_price, max_ratio_pc, boost_ratio_pc
        );
        println!("net value:");
        println!("{:>15.2} dai", dai_value);
        println!("{:>15.2} eur", eur_value);
        println!("{:>15.2} btc", btc_value);
        println!("{:>15.2} eth", col_value);
        Ok(())
    }
}
