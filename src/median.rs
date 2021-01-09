use super::blockchain::BlockchainReader;
use ethabi::{Address, Uint};
use std::error::Error;

pub struct Median<'a> {
    blockchain_reader: &'a dyn BlockchainReader,
    median_address: Address,
}

impl<'a> Median<'a> {
    pub fn new(
        blockchain_reader: &'a (dyn BlockchainReader + 'a),
        median_address: Address,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            blockchain_reader,
            median_address,
        })
    }

    pub async fn get_price(&self, next: bool) -> Result<Uint, Box<dyn Error>> {
        let position = match next {
            true => Uint::from(4),
            false => Uint::from(3),
        };
        let data: Vec<u8> = self
            .blockchain_reader
            .get_storage_at(&self.median_address, position)
            .await?;
        let data: Vec<u8> = data.iter().rev().take(16).rev().cloned().collect();
        Ok(Uint::from_big_endian(&data))
    }
}
