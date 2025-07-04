use anchor_lang::prelude::*;
use anchor_lang::prelude::Pubkey;

pub const CURRENT_VERSION: u8 = 1;
pub const DATA_LINK_SIZE: usize = 256;
pub const PRIMARY_CATEGORY_SIZE: usize = 32;
pub const SECONDARY_CATEGORY_SIZE: usize = 32;
pub const RESERVED_SIZE: usize = 128;

#[account]
#[derive(Eq, PartialEq, Debug)]
pub struct AgentResponse {
    pub agent_key: Pubkey,
    pub response: bool,
    pub rating: u8,
    pub calculated_credits: u128,
}

impl AgentResponse {
    pub const MAX_SIZE: usize = 1 + 32 + 1 + 16;
}

#[account]
#[derive(Eq, PartialEq, Debug)]
pub struct DataHeader {
    pub primary_category: [u8; PRIMARY_CATEGORY_SIZE],
    pub secondary_category: [u8; SECONDARY_CATEGORY_SIZE],
    pub reserved: [u8; 128]
}

impl DataHeader {
    pub const MAX_SIZE: usize = PRIMARY_CATEGORY_SIZE + SECONDARY_CATEGORY_SIZE + RESERVED_SIZE;
}

#[account]
#[derive(Eq, PartialEq, Debug)]
pub struct Datasubmission {
    pub version: u8,
    pub data_link: [u8; DATA_LINK_SIZE],
    pub agent_response: Option<AgentResponse>,
    pub data_header: DataHeader
}

impl Datasubmission {
    pub const MAX_SIZE: usize = 1 + DATA_LINK_SIZE + 1 + AgentResponse::MAX_SIZE + DataHeader::MAX_SIZE;
}

#[account]
pub struct ProgramState {
    pub version: u8,
    pub admin: Pubkey,
}

impl ProgramState {
    pub const MAX_SIZE: usize = 1 + 32;
}

#[derive(Eq, PartialEq, Debug)]
#[account]
pub struct AgentConfig {
    pub version: u8,
    pub is_enabled: bool,
}

impl AgentConfig {
    pub const MAX_SIZE: usize = 2;
}

