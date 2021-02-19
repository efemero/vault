use ethabi::Address;

#[derive(Debug, Clone)]
pub struct Erc20Token {
    pub name: String,
    pub symbol: String,
    pub decimals: usize,
    pub address: Address,
}
