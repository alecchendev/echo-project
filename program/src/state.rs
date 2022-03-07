use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AuthorizedBufferHeader {
    // TODO
    pub bump: u8,
    pub buffer_seed: u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VendingMachineBufferHeader {
    // TODO
    pub bump: u8,
    pub price: u64
}
