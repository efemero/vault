#[macro_use]
extern crate clap;
use async_jsonrpc_client::HttpTransport;
use cli_table::{format::Justify, print_stdout, Cell, Style, Table};
use ethabi::{Address, Uint};
use std::error::Error;
use vault::{
    get_simulation, CdpManager, HttpBlockchainReader, Median, Pair, Saver, Scenario, Spot,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(
        vault =>
        (version: "0.3.0")
        (author: "François Bastien <fmrbastien@gmail.com>")
        (about: "Get informations about your makerDAO vault.")
        (@subcommand show =>
         (@arg friction: -f --friction +takes_value  default_value("0.03") "The friction takes in account the transactions fees, and market friction." )
         (@arg NODE: -n --node +takes_value default_value("localhost:8545") "Ethereum node to call" )
         (@arg VAULT_ID: +required "The ID of the vault to check" )
         (about: "display the vault state for the choosen price")
         (@arg next: --next "if present, computations are based on the next price" )
         (@arg price: -p --price +takes_value  "if present, computations are based on this price (DAI / ETH)" )
         (@arg max_ratio: --max_ratio +takes_value  "set this to override the max_ratio of the current vault (in %)" )
         (@arg boost_ratio: --boost_ratio +takes_value  "set this to override the boost_ratio of the current vault (in %)" )
         (@arg min_ratio: --min_ratio +takes_value  "set this to override the min_ratio of the current vault (in %)" )
         (@arg repay_ratio: --repay_ratio +takes_value  "set this to override the repay_ratio of the current vault (in %)" )
        )
        (@subcommand optimize =>
         (about: "launch a serie of simulations to choose the best ratios")
         (@arg friction: -f --friction +takes_value  default_value("0.03") "The friction takes in account the transactions fees, and market friction." )
         (@arg increase: --increase +takes_value  default_value("10") "The price increase to simulate." )
         (@arg start:  --start +takes_value  default_value("180") "The ratio at the start of the simulation (in %)." )
         (@arg end:  --end +takes_value  default_value("250") "The ratio at the end of the simulation (in %)." )
         (@arg list: --list "if present, show the result as a list (default to table)" )
        )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("show") {
        let friction = value_t_or_exit!(matches.value_of("friction"), f64);
        let vault_id = value_t_or_exit!(matches.value_of("VAULT_ID"), u128);
        let node = matches.value_of("NODE").unwrap();
        let transport = HttpTransport::new(node);
        let reader: HttpBlockchainReader = HttpBlockchainReader::new(transport)?;
        let saver = Saver::new(&reader)?;
        let cdp_manager: CdpManager = CdpManager::new(&reader)?;
        let vault_id = Uint::from(vault_id);
        let price;
        let wbtc_eth_pair_address: Address = "Bb2b8038a1640196FbE3e38816F3e67Cba72D940".parse()?;
        let wbtc_eth_pair = Pair::new(&reader, wbtc_eth_pair_address)?;
        let wbtc_price = wbtc_eth_pair.get_price_0().await?;

        if matches.is_present("price") {
            let price_f64 = value_t_or_exit!(matches.value_of("price"), f64);
            price = Uint::from((price_f64 * 1000.0) as i64) * Uint::exp10(15);
        } else {
            let spot: Spot = Spot::new(&reader)?;
            let next = matches.is_present("next");
            let ilk_id = cdp_manager.get_ilk_id(vault_id).await?;
            let median_address = spot.get_median_address(&ilk_id).await?;
            let median = Median::new(&reader, median_address)?;
            price = median.get_price(next).await?
        }
        let mut vault = saver.get_vault(vault_id).await?;
        if matches.is_present("max_ratio") {
            let max_ratio_pc = value_t_or_exit!(matches.value_of("max_ratio"), usize);
            let max_ratio = Uint::from(max_ratio_pc) * Uint::exp10(16);
            vault.max_ratio = max_ratio;
        }
        if matches.is_present("boost_ratio") {
            let boost_ratio_pc = value_t_or_exit!(matches.value_of("boost_ratio"), usize);
            let boost_ratio = Uint::from(boost_ratio_pc) * Uint::exp10(16);
            vault.boost_ratio = boost_ratio;
        }
        if matches.is_present("min_ratio") {
            let min_ratio_pc = value_t_or_exit!(matches.value_of("min_ratio"), usize);
            let min_ratio = Uint::from(min_ratio_pc) * Uint::exp10(16);
            vault.min_ratio = min_ratio;
        }
        if matches.is_present("repay_ratio") {
            let repay_ratio_pc = value_t_or_exit!(matches.value_of("repay_ratio"), usize);
            let repay_ratio = Uint::from(repay_ratio_pc) * Uint::exp10(16);
            vault.repay_ratio = repay_ratio;
        }
        vault = vault.predict_vault(price, friction)?;
        vault.show(price, wbtc_price)?;
    } else if let Some(matches) = matches.subcommand_matches("optimize") {
        let friction = value_t_or_exit!(matches.value_of("friction"), f64);
        let increase = value_t_or_exit!(matches.value_of("increase"), f64);
        let start = value_t_or_exit!(matches.value_of("start"), usize);
        let end = value_t_or_exit!(matches.value_of("end"), usize);
        let list = matches.is_present("list");
        let mut scenarios = Vec::with_capacity(end - start + 1);
        for boost in start..=end {
            let scenario = get_simulation(boost, increase, friction, 140)?;
            scenarios.push(scenario);
        }
        print_scenarios(scenarios, !list);
    } else {
        println!("{}", matches.usage());
    }

    Ok(())
}

fn print_scenarios(scenarios: Vec<Scenario>, table: bool) {
    if table {
        let mut vecs = Vec::with_capacity(scenarios.len());
        let increase = &scenarios[0].clone().price_increase;
        for scenario in scenarios {
            vecs.push(vec![
                scenario.boost_ratio.cell(),
                format!("x{:.2}", scenario.no_boost_increase)
                    .cell()
                    .justify(Justify::Right),
                format!("x{:.2}", scenario.best_result.increase)
                    .cell()
                    .justify(Justify::Right),
                format!("{}", scenario.best_result.max_ratio)
                    .cell()
                    .justify(Justify::Right),
            ])
        }

        println!("What happens when the price makes x{:.2}?", increase);
        println!("If you hodl, you will make x{:.2}.", increase);
        let table = vecs
            .table()
            .title(vec![
                "CDP ratio".cell().bold(true),
                "No boost".cell().bold(true),
                "Best boost".cell().bold(true),
                "Best ratio".cell().bold(true),
            ])
            .bold(true);

        assert!(print_stdout(table).is_ok());
    } else {
        for scenario in scenarios {
            println!("{}", scenario);
        }
    }
}
