use super::blockchain::BlockchainReader;
use ethabi::{Address, Contract, Token, Uint};
use std::error::Error;

const CDP_MANAGER_ADDRESS: &str = "5ef30b9986345249bc32d8928b7ee64de9435e39";
pub struct CdpManager<'a> {
    blockchain_reader: &'a dyn BlockchainReader,
    cdp_manager_address: Address,
    cdp_manager_contract: Contract,
}
impl<'a> CdpManager<'a> {
    pub fn new(blockchain_reader: &'a (dyn BlockchainReader + 'a)) -> Result<Self, Box<dyn Error>> {
        let cdp_manager_address: Address = CDP_MANAGER_ADDRESS.parse()?;
        let cdp_manager_abi: &[u8] = include_bytes!("abi/cdp_manager.abi");
        let cdp_manager_contract: Contract = Contract::load(cdp_manager_abi)?;
        Ok(Self {
            blockchain_reader,
            cdp_manager_address,
            cdp_manager_contract,
        })
    }
    pub async fn get_ilk_id(&self, vault_id: Uint) -> Result<Vec<u8>, Box<dyn Error>> {
        let tokens = self
            .blockchain_reader
            .call_function(
                &self.cdp_manager_contract,
                &self.cdp_manager_address,
                "ilks",
                &[Token::Uint(vault_id)],
            )
            .await?;

        let ilk_id = tokens[0].clone().to_fixed_bytes();
        let ilk_id = ilk_id.unwrap();
        Ok(ilk_id)
    }
}
