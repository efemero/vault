use super::blockchain::BlockchainReader;
use super::erc_20::Erc20Token;
use super::vault::Vault;
use ethabi::{Address, Contract, Token, Uint};
use std::error::Error;

const SAVER_ADDRESS: &str = "c45d4f6b6bf41b6edaa58b01c4298b8d9078269a";
pub struct Saver<'a> {
    blockchain_reader: &'a dyn BlockchainReader,
    saver_address: Address,
    saver_contract: Contract,
}
impl<'a> Saver<'a> {
    pub fn new(blockchain_reader: &'a (dyn BlockchainReader + 'a)) -> Result<Self, Box<dyn Error>> {
        let saver_address: Address = SAVER_ADDRESS.parse()?;
        let saver_abi: &[u8] = include_bytes!("abi/saver.abi");
        let saver_contract = Contract::load(saver_abi)?;

        let reader = Self {
            blockchain_reader,
            saver_address,
            saver_contract,
        };
        Ok(reader)
    }

    pub async fn get_vault(&self, vault_id: Uint) -> Result<Vault, Box<dyn Error>> {
        let tokens = self
            .blockchain_reader
            .call_function(
                &self.saver_contract,
                &self.saver_address,
                "getSubscribedInfo",
                &[Token::Uint(vault_id)],
            )
            .await?;

        let subscribed = &tokens[0].clone().to_bool();
        let min_ratio = &tokens[1].clone().to_uint();
        let max_ratio = &tokens[2].clone().to_uint();
        let repay_ratio = &tokens[3].clone().to_uint();
        let boost_ratio = &tokens[4].clone().to_uint();
        let owner = &tokens[5].clone().to_address();
        let collateral = &tokens[6].clone().to_uint();
        let debt = &tokens[7].clone().to_uint();

        let vault = Vault {
            subscribed: subscribed.unwrap(),
            //min_ratio: Uint::from(160) * Uint::exp10(16),
            //max_ratio: Uint::from(220) * Uint::exp10(16),
            //repay_ratio: Uint::from(180) * Uint::exp10(16),
            //boost_ratio: Uint::from(180) * Uint::exp10(16),
            min_ratio: min_ratio.unwrap(),
            max_ratio: max_ratio.unwrap(),
            repay_ratio: repay_ratio.unwrap(),
            boost_ratio: boost_ratio.unwrap(),
            owner: owner.unwrap(),
            collateral: collateral.unwrap(),
            debt: debt.unwrap(),
            token: Erc20Token {
                name: "Wrapped Ether".to_string(),
                symbol: "WETH".to_string(),
                decimals: 18,
                address: "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".parse()?,
            },
        };
        Ok(vault)
    }
}
