use super::blockchain::BlockchainReader;
use ethabi::{Address, Contract, Token};
use std::error::Error;

const SPOT_ADDRESS: &str = "65c79fcb50ca1594b025960e539ed7a9a6d434a3";

pub struct Spot<'a> {
    blockchain_reader: &'a dyn BlockchainReader,
    spot_address: Address,
    spot_contract: Contract,
}
impl<'a> Spot<'a> {
    pub fn new(blockchain_reader: &'a (dyn BlockchainReader + 'a)) -> Result<Self, Box<dyn Error>> {
        let spot_address: Address = SPOT_ADDRESS.parse()?;
        let spot_abi: &[u8] = include_bytes!("abi/spot.abi");
        let spot_contract = Contract::load(spot_abi)?;
        Ok(Self {
            blockchain_reader,
            spot_address,
            spot_contract,
        })
    }

    pub async fn get_median_address(&self, ilk_id: &Vec<u8>) -> Result<Address, Box<dyn Error>> {
        let tokens = self
            .blockchain_reader
            .call_function(
                &self.spot_contract,
                &self.spot_address,
                "ilks",
                &[Token::FixedBytes(ilk_id.to_vec())],
            )
            .await?;

        let median_address = tokens[0].clone().to_address();
        Ok(median_address.unwrap())
    }
}
