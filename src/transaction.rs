use ethers::types::{Address, U256};

pub trait Transactionable: Sized + Sync + Send {
    fn calldata(&self) -> Option<&[u8]>;
    fn to(&self) -> &Address;
    fn value(&self) -> &U256;
}
