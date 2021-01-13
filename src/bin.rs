#[macro_use]
extern crate clap;
use async_jsonrpc_client::HttpTransport;
use ethabi::{Address, Uint};
use std::error::Error;
use vault::{CdpManager, HttpBlockchainReader, Median, Pair, Saver, Spot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(
        myapp =>
        (version: "0.2")
        (author: "François Bastien <fmrbastien@gmail.com>")
        (about: "Get informations about your makerDAO vault.")
        (@arg NODE: -n --node +takes_value default_value("localhost:8545") "Ethereum node to call" )
        (@arg next: --next "if present, computations are based on the next price" )
        (@arg price: -p --price +takes_value  "if present, computations are based on this price (DAI / ETH)" )
        (@arg friction: -f --friction +takes_value  default_value("0.03") "The friction takes in account the transactions fees, and market friction." )
        (@arg VAULT_ID: +required "The ID of the vault to check" )
        )
        .get_matches();

    let vault_id = value_t_or_exit!(matches.value_of("VAULT_ID"), u128);
    let node = matches.value_of("NODE").unwrap();

    let transport = HttpTransport::new(node);
    let reader: HttpBlockchainReader = HttpBlockchainReader::new(transport)?;
    let saver = Saver::new(&reader)?;
    let cdp_manager: CdpManager = CdpManager::new(&reader)?;
    let vault_id = Uint::from(vault_id);
    let price;
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
    let wbtc_eth_pair_address: Address = "Bb2b8038a1640196FbE3e38816F3e67Cba72D940".parse()?;
    let wbtc_eth_pair = Pair::new(&reader, wbtc_eth_pair_address)?;
    let wbtc_price = wbtc_eth_pair.get_price_0().await?;

    let friction = value_t_or_exit!(matches.value_of("friction"), f64);
    let vault = saver
        .get_vault(vault_id)
        .await?
        .predict_vault(price, friction)?;
    vault.show(price, wbtc_price)?;

    Ok(())
}
