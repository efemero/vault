use super::blockchain::BlockchainReader;
use ethabi::{Address, Contract, Token, Uint};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Erc20Token {
    pub name: String,
    pub symbol: String,
    pub decimals: usize,
    pub address: Address,
}

pub struct Erc20TokenReader<'a> {
    blockchain_reader: &'a dyn BlockchainReader,
    token_address: Address,
    erc20_contract: Contract,
    erc20_bytes_contract: Contract,
}

impl<'a> Erc20TokenReader<'a> {
    async fn get_name(&self, contract_address: &Address) -> Result<String, Box<dyn Error>> {
        let tokens = self
            .blockchain_reader
            .call_function(&self.erc20_contract, contract_address, "name", &[])
            .await;
        let name: String = match tokens {
            Ok(tokens) => tokens[0]
                .clone()
                .to_string()
                .ok_or(ArbiswapError(String::from("cannot get name from string")))?,
            _ => {
                let token = &self
                    .blockchain_reader
                    .call_function(&self.erc20_bytes_contract, contract_address, "name", &[])
                    .await?[0];
                String::from_utf8(
                    token
                        .clone()
                        .to_fixed_bytes()
                        .ok_or(ArbiswapError(String::from("cannot get name from bytes")))?,
                )?
                .chars()
                .filter_map(|x| match x {
                    '\0' => None,
                    _ => Some(x),
                })
                .collect::<String>()
            }
        };
        Ok(name)
    }

    async fn get_symbol(&self, contract_address: &Address) -> Result<String, Box<dyn Error>> {
        let tokens = self
            .blockchain_reader
            .call_function(&self.erc20_contract, contract_address, "symbol", &[])
            .await;
        let symbol: String = match tokens {
            Ok(tokens) => tokens[0]
                .clone()
                .to_string()
                .ok_or(ArbiswapError(String::from("cannot get symbol from string")))?,
            _ => {
                let token = &self
                    .blockchain_reader
                    .call_function(&self.erc20_bytes_contract, contract_address, "symbol", &[])
                    .await?[0];
                String::from_utf8(
                    token
                        .clone()
                        .to_fixed_bytes()
                        .ok_or(ArbiswapError(String::from("cannot get symbol from bytes")))?,
                )?
                .chars()
                .filter_map(|x| match x {
                    '\0' => None,
                    _ => Some(x),
                })
                .collect::<String>()
            }
        };
        Ok(symbol)
    }

    async fn get_decimals(&self, contract_address: &Address) -> Result<u8, Box<dyn Error>> {
        let tokens = self
            .blockchain_reader
            .call_function(&self.erc20_contract, contract_address, "decimals", &[])
            .await?;
        let result: u8 = tokens[0]
            .clone()
            .to_uint()
            .ok_or(ArbiswapError(String::from(
                "cannot get hex string for decimals",
            )))?
            .as_u32() as u8;
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct ArbiswapError(pub String);

impl fmt::Display for ArbiswapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for ArbiswapError {
    fn description(&self) -> &str {
        &self.0
    }
}
