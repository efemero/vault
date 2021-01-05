use async_jsonrpc_client::HttpTransport;
use async_jsonrpc_client::{Params, Transport, Value};
use async_trait::async_trait;
use ethabi::{Address, Contract, Token, Uint};
use rustc_hex::FromHex;
use serde_json::json;
use std::error::Error;
use std::fmt;
use web3::types::Bytes;
use web3::types::CallRequest;

const SAVER_ADDRESS: &str = "c45d4f6b6bf41b6edaa58b01c4298b8d9078269a";
const CDP_MANAGER_ADDRESS: &str = "5ef30b9986345249bc32d8928b7ee64de9435e39";
const SPOT_ADDRESS: &str = "65c79fcb50ca1594b025960e539ed7a9a6d434a3";

#[derive(Debug, Clone)]
pub struct Vault {
    subscribed: bool,
    min_ratio: Uint,
    max_ratio: Uint,
    repay_ratio: Uint,
    boost_ratio: Uint,
    owner: Address,
    col: Uint,
    debt: Uint,
}

impl Vault {
    pub fn get_dai_value(&self, price: Uint) -> Result<Uint, Box<dyn Error>> {
        let dai_col = price * self.col / Uint::exp10(18);
        Ok(dai_col - self.debt)
    }
    pub fn get_col_value(&self, price: Uint) -> Result<Uint, Box<dyn Error>> {
        let dai_value = self.get_dai_value(price)?;
        Ok(dai_value * Uint::exp10(18) / price)
    }
}

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
        let col = &tokens[6].clone().to_uint();
        let debt = &tokens[7].clone().to_uint();

        let vault = Vault {
            subscribed: subscribed.unwrap(),
            min_ratio: min_ratio.unwrap(),
            max_ratio: max_ratio.unwrap(),
            repay_ratio: repay_ratio.unwrap(),
            boost_ratio: boost_ratio.unwrap(),
            owner: owner.unwrap(),
            col: col.unwrap(),
            debt: debt.unwrap(),
        };
        Ok(vault)
    }
}

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
        Ok(ilk_id.unwrap())
    }
}

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

        let spot_address = tokens[0].clone().to_address();
        Ok(spot_address.unwrap())
    }
}

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

#[async_trait]
pub trait BlockchainReader {
    async fn call_function(
        &self,
        contract: &Contract,
        contract_address: &Address,
        name: &str,
        params: &[Token],
    ) -> Result<Vec<Token>, Box<dyn Error>>;

    async fn get_storage_at(
        &self,
        address: &Address,
        position: Uint,
    ) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct HttpBlockchainReader {
    transport: HttpTransport,
}

impl HttpBlockchainReader {
    pub fn new(transport: HttpTransport) -> Result<Self, Box<dyn Error>> {
        Ok(Self { transport })
    }
}

#[async_trait]
impl BlockchainReader for HttpBlockchainReader {
    async fn call_function(
        &self,
        contract: &Contract,
        contract_address: &Address,
        name: &str,
        params: &[Token],
    ) -> Result<Vec<Token>, Box<dyn Error>> {
        let function = contract.function(name)?;
        let data = function.encode_input(params)?;

        let req = serde_json::to_value(CallRequest {
            from: None,
            to: Some(*contract_address),
            gas: None,
            gas_price: None,
            value: None,
            data: Some(Bytes(data)),
        })?;
        let params = Params::Array(vec![req, json!("latest")]);
        let response: Value = self.transport.send("eth_call", params).await?;
        let hex_str = &response.as_str().ok_or(VaultError(String::from(
            "cannot retrieve response from eth_call",
        )))?[2..];
        let data: Vec<u8> = hex_str.from_hex()?;
        let result = function.decode_output(&data)?;
        Ok(result)
    }

    async fn get_storage_at(
        &self,
        address: &Address,
        position: Uint,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let params = Params::Array(vec![
            Value::String(format!("{:#x}", address)),
            Value::String(format!("{:#x}", position)),
            Value::String("latest".to_string()),
        ]);
        let response: Value = self.transport.send("eth_getStorageAt", params).await?;
        let hex_str = &response.as_str().ok_or(VaultError(String::from(
            "cannot retrieve response from eth_call",
        )))?[2..];
        let data: Vec<u8> = hex_str.from_hex()?;
        let data: Vec<u8> = data.iter().rev().take(16).rev().cloned().collect();
        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub struct VaultError(pub String);

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for VaultError {
    fn description(&self) -> &str {
        &self.0
    }
}
