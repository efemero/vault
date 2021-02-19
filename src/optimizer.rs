use super::erc_20::Erc20Token;
use super::vault::Vault;
use ethabi::Uint;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub max_ratio: usize,
    pub increase: f64,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub results: Vec<ScenarioResult>,
    pub best_result: ScenarioResult,
    pub boost_ratio: usize,
    pub price_increase: f64,
    pub no_boost_increase: f64,
    pub friction: f64,
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "No CDP: x{:>5.2}, CDP at {}%: x{:>5.2}, CDP boosted {}% -> {}%: x{:>5.2}",
            self.price_increase,
            self.best_result.max_ratio,
            self.no_boost_increase,
            self.best_result.max_ratio,
            self.boost_ratio,
            self.best_result.increase
        )
    }
}

pub fn get_simulation(
    boost_ratio: usize,
    price_increase: f64,
    friction: f64,
    end: usize,
) -> Result<Scenario, Box<dyn Error>> {
    let mut scenario = Scenario {
        results: Vec::with_capacity(end),
        best_result: ScenarioResult {
            max_ratio: boost_ratio,
            increase: 0.0,
        },
        boost_ratio,
        price_increase,
        no_boost_increase: 0.0,
        friction,
    };
    let mut vault = Vault {
        subscribed: true,
        min_ratio: Uint::from(100) * Uint::exp10(16),
        max_ratio: Uint::from(220) * Uint::exp10(16),
        repay_ratio: Uint::from(180) * Uint::exp10(16),
        boost_ratio: Uint::from(180) * Uint::exp10(16),
        owner: "0000000000000000000000000000000000000000".parse()?,
        collateral: Uint::from(180) * Uint::exp10(18),
        debt: Uint::from(10000) * Uint::exp10(18),
        token: Erc20Token {
            name: "".to_string(),
            symbol: "".to_string(),
            decimals: 18,
            address: "0000000000000000000000000000000000000000".parse()?,
        },
    };

    if end < 1 {
        return Ok(scenario);
    }
    let base_price = 100;
    let start_price = Uint::from(base_price) * Uint::exp10(18);
    let up_price = Uint::from((start_price.as_u128() as f64 * price_increase) as u128);
    vault.boost_ratio = Uint::from(boost_ratio) * Uint::exp10(16);
    vault.collateral = vault.debt * vault.boost_ratio / start_price;
    let base = vault.get_dai_value(start_price)?;
    let base_up = vault.get_dai_value(up_price)?;
    for max_ratio in (boost_ratio + 1)..=(boost_ratio + end) {
        vault.max_ratio = Uint::from(max_ratio) * Uint::exp10(16);
        let r = vault.predict_vault(up_price, friction);
        let dai_value = match r {
            Ok(v2) => v2.get_dai_value(up_price)?,
            _ => Uint::zero(),
        };
        let increase = dai_value.as_u128() as f64 / base.as_u128() as f64;
        let scenario_result = ScenarioResult {
            max_ratio,
            increase,
        };
        scenario.results.push(scenario_result.clone());
        if increase > scenario.best_result.increase {
            scenario.best_result = scenario_result;
        }
    }
    scenario.no_boost_increase = base_up.as_u128() as f64 / base.as_u128() as f64;
    Ok(scenario)
}
