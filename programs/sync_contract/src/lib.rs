pub mod types;

use crate::types::{AgentConfig, Datasubmission, ProgramState};
use anchor_lang::prelude::*;

declare_id!("HkUDiDMSDntG1p4CgEaT9nhVrZ6MpPTVyL8rYiX3nvxy");

use anchor_lang::error_code;
use anchor_lang::solana_program::keccak;

#[error_code]
pub enum SyncError {
    #[msg("The caller is not administrator")]
    ErrCallerNotAdmin,
    #[msg("Invalid data link")]
    ErrInvalidDataLink,
    #[msg("Data link is empty")]
    ErrDataLinkEmpty,
    #[msg("Data link is too large")]
    ErrDataLinkTooLarge,
    #[msg("Data is already rated")]
    ErrDataAlreadyRated,
    #[msg("Agent is not enabled")]
    ErrAgentIsNotEnabled,
    #[msg("Invalid rating")]
    ErrInvalidRating,
    #[msg("Primary category is too large")]
    ErrPrimaryCategoryTooLarge,
    #[msg("Secondary category is too large")]
    ErrSecondaryCategoryTooLarge,
}

pub fn calculate_credits(rating: u8) -> u128 {
    if rating >= 80 {
        rating as u128
    } else {
        0
    }
}

#[program]
pub mod sync_contract {
    use crate::types::{AgentConfig, AgentResponse, DataHeader, ProgramState, CURRENT_VERSION, DATA_LINK_SIZE, PRIMARY_CATEGORY_SIZE, RESERVED_SIZE, SECONDARY_CATEGORY_SIZE};
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        *ctx.accounts.program_state = ProgramState {
            version: CURRENT_VERSION,
            admin: ctx.accounts.signer.key.clone(),
        };
        Ok(())
    }
    
    pub fn create_agent(ctx: Context<CreateAgent>) -> Result<()> {
        *ctx.accounts.agent_config = AgentConfig {
            version: CURRENT_VERSION,
            is_enabled: false,
        };
        Ok(())
    }
    
    pub fn allow_agent(ctx: Context<AllowAgent>, agent_key: Pubkey) -> Result<()> {
        ctx.accounts.agent_config.is_enabled = true;
        Ok(())
    }
    
    pub fn submit_data(ctx: Context<SubmitData>, data_link: String, primary_category: String, secondary_category: String) -> Result<()> {
        if data_link.is_empty() {
            return Err(error!(SyncError::ErrDataLinkEmpty));
        }
        
        if !data_link.is_ascii() {
            return Err(error!(SyncError::ErrInvalidDataLink));
        }
        
        let link_bytes = data_link.as_bytes();
        let mut storage_link_bytes = [0u8; DATA_LINK_SIZE]; 
        if link_bytes.len() > DATA_LINK_SIZE {
            return Err(error!(SyncError::ErrDataLinkTooLarge));
        }
        storage_link_bytes[0..link_bytes.len()].copy_from_slice(link_bytes);

        let primary_category_string_bytes = primary_category.as_bytes();
        if primary_category_string_bytes.len() > PRIMARY_CATEGORY_SIZE {
            return Err(error!(SyncError::ErrPrimaryCategoryTooLarge));
        }
        let mut primary_category_bytes = [0u8; PRIMARY_CATEGORY_SIZE];
        primary_category_bytes[0..primary_category_string_bytes.len()].copy_from_slice(primary_category_string_bytes);

        let secondary_category_string_bytes = secondary_category.as_bytes();
        if secondary_category_string_bytes.len() > SECONDARY_CATEGORY_SIZE {
            return Err(error!(SyncError::ErrSecondaryCategoryTooLarge));
        }
        let mut secondary_category_bytes = [0u8; SECONDARY_CATEGORY_SIZE];
        secondary_category_bytes[0..secondary_category_string_bytes.len()].copy_from_slice(secondary_category_string_bytes);

        *ctx.accounts.data_submission = Datasubmission {
            version: CURRENT_VERSION,
            data_link: storage_link_bytes,
            agent_response: None,
            data_header: DataHeader {
                primary_category: primary_category_bytes,
                secondary_category: secondary_category_bytes,
                reserved: [0u8; RESERVED_SIZE],
            },
        };
        
        Ok(())
    }
    
    pub fn rate_data(ctx: Context<RateData>, data_link: String, passed: bool, rating: u8) -> Result<()> {
        if !ctx.accounts.agent_config.is_enabled {
            return Err(error!(SyncError::ErrAgentIsNotEnabled));
        }
        
        if ctx.accounts.data_submission.agent_response.is_some() {
            return Err(error!(SyncError::ErrDataAlreadyRated));
        }
        
        if rating > 100 {
            return Err(error!(SyncError::ErrInvalidRating));
        }
        
        ctx.accounts.data_submission.agent_response = Some(AgentResponse {
            agent_key: ctx.accounts.signer.key.clone(),
            response: passed,
            rating: rating,
            calculated_credits: calculate_credits(rating),
        });
        Ok(())
    }
}

fn hash_data_link(data_link: &String) -> [u8; 32] {
    keccak::hash(data_link.as_bytes()).to_bytes()
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [b"sync_program".as_ref(), b"global_state".as_ref()],
        bump,
        space = 8 + ProgramState::MAX_SIZE
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateAgent<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [b"sync_program".as_ref(), b"agent_config".as_ref(), signer.key().as_ref()],
        bump,
        space = 8 + AgentConfig::MAX_SIZE
    )]
    pub agent_config: Account<'info, AgentConfig>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(agent_key: Pubkey)]
pub struct AllowAgent<'info> {
    #[account(
        seeds = [b"sync_program".as_ref(), b"global_state".as_ref()],
        bump,
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut, address = program_state.admin)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"sync_program".as_ref(), b"agent_config".as_ref(), agent_key.as_ref()],
        bump,
    )]
    pub agent_config: Account<'info, AgentConfig>,
}

#[derive(Accounts)]
#[instruction(data_link: String)]
pub struct SubmitData<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [b"sync_program".as_ref(), b"data_submission".as_ref(), &hash_data_link(&data_link)],
        bump,
        space = 8 + Datasubmission::MAX_SIZE
    )]
    pub data_submission: Account<'info, Datasubmission>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(data_link: String)]
pub struct RateData<'info> {
    #[account(
        mut,
        seeds = [b"sync_program".as_ref(), b"data_submission".as_ref(), &hash_data_link(&data_link)],
        bump,
    )]
    pub data_submission: Account<'info, Datasubmission>,
    #[account(
        seeds = [b"sync_program".as_ref(), b"agent_config".as_ref(), signer.key().as_ref()],
        bump,
    )]
    pub agent_config: Account<'info, AgentConfig>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

