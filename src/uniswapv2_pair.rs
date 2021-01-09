use super::blockchain::BlockchainReader;
use ethabi::{Address, Contract, Uint};
use std::error::Error;

pub struct Pair<'a> {
    blockchain_reader: &'a dyn BlockchainReader,
    pair_address: Address,
    pair_contract: Contract,
}

impl<'a> Pair<'a> {
    pub fn new(
        blockchain_reader: &'a (dyn BlockchainReader + 'a),
        pair_address: Address,
    ) -> Result<Self, Box<dyn Error>> {
        let pair_abi: &[u8] = include_bytes!("abi/uniswapv2_pair.abi");
        let pair_contract: Contract = Contract::load(pair_abi)?;
        Ok(Self {
            blockchain_reader,
            pair_address,
            pair_contract,
        })
    }

    pub async fn get_price_0(&self) -> Result<f64, Box<dyn Error>> {
        let tokens = self
            .blockchain_reader
            .call_function(&self.pair_contract, &self.pair_address, "getReserves", &[])
            .await?;

        let reserve0 = tokens[0].clone().to_uint();
        let reserve0 = reserve0.unwrap();
        let reserve1 = tokens[1].clone().to_uint();
        let reserve1 = reserve1.unwrap();

        let price0 = Uint::exp10(10).as_u128() as f64 * reserve0.as_u128() as f64
            / reserve1.as_u128() as f64;
        Ok(price0)
    }
}
