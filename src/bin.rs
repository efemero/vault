#[macro_use]
extern crate clap;
use async_jsonrpc_client::HttpTransport;
use ethabi::Uint;
use std::error::Error;
use vault::{CdpManager, HttpBlockchainReader, Median, Saver, Spot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(
        myapp =>
        (version: "0.1")
        (author: "François Bastien <fmrbastien@gmail.com>")
        (about: "Get informations about your makerDAO vault.")
        (@arg NODE: -n --node +takes_value default_value("localhost:8545") "Ethereum node to call" )
        (@arg next: --next "if present, computations are based on the next price" )
        (@arg VAULT_ID: +required "The ID of the vault to check" )
        (@arg ceiling: -c --ceiling "the highest price to predict (in dai), defaults to the current price * 10")
        (@arg floor: -f --floor "the lowest price to predict (in dai), defaults to the current price / 10")
        )
        .get_matches();

    let vault_id = value_t_or_exit!(matches.value_of("VAULT_ID"), u128);
    let node = matches.value_of("NODE").unwrap();
    let next = matches.is_present("next");
    let transport = HttpTransport::new(node);
    let reader: HttpBlockchainReader = HttpBlockchainReader::new(transport)?;
    let saver = Saver::new(&reader)?;
    let cdp_manager: CdpManager = CdpManager::new(&reader)?;
    let spot: Spot = Spot::new(&reader)?;
    let vault_id = Uint::from(vault_id);
    let vault = saver.get_vault(vault_id).await?;
    let ilk_id = cdp_manager.get_ilk_id(vault_id).await?;
    let median_address = spot.get_median_address(&ilk_id).await?;
    let median = Median::new(&reader, median_address)?;
    let price = median.get_price(next).await?;
    let dai_value = vault.get_dai_value(price)?;
    let col_value = vault.get_col_value(price)?;
    println!(
        "price: {}",
        price.as_u128() as f64 / Uint::exp10(18).as_u128() as f64
    );
    println!("net value:");
    println!(
        "\t{} dai",
        dai_value.as_u128() as f64 / Uint::exp10(18).as_u128() as f64
    );
    println!(
        "\t{} eth",
        col_value.as_u128() as f64 / Uint::exp10(18).as_u128() as f64
    );
    Ok(())
}
