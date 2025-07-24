pub mod types;

use crate::types::{
    AgentConfig, AgentResponse, DataHeader, Datasubmission, ProgramState, UserConfig,
    CURRENT_VERSION, DATAHEADER_RESERVED_SIZE, DATA_LINK_SIZE, PRIMARY_CATEGORY_SIZE,
    SECONDARY_CATEGORY_SIZE, USER_CONFIG_RESERVED_SIZE,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, MintTo, SetAuthority, TokenAccount, TokenInterface};

declare_id!("HkUDiDMSDntG1p4CgEaT9nhVrZ6MpPTVyL8rYiX3nvxy");

use anchor_lang::error_code;
use anchor_lang::solana_program::keccak;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::spl_token_2022;
#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

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
    if rating >= 60 {
        rating as u128
    } else {
        0
    }
}

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name:  "Syncora.ai Contract",
    project_url:  "https://syncora.ai",
    contacts:  "email:support@syncora.ai",
    policy:  "https://www.syncora.ai/privacyPolicy",
    preferred_languages: "en",
    source_code: ""
}

#[program]
pub mod sync_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        *ctx.accounts.program_state = ProgramState {
            version: CURRENT_VERSION,
            admin: ctx.accounts.signer.key.clone(),
        };
        Ok(())
    }

    pub fn transfer_mint_authority(
        ctx: Context<TransferMintAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        if *ctx.accounts.signer.key != ctx.accounts.program_state.admin {
            return Err(error!(SyncError::ErrCallerNotAdmin));
        }

        let cpi_accounts = SetAuthority {
            current_authority: ctx.accounts.program_state.to_account_info(),
            account_or_mint: ctx.accounts.mint.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let signer_seeds = [
            b"sync_program".as_ref(),
            b"global_state".as_ref(),
            &[ctx.bumps.program_state],
        ];
        let binding = [signer_seeds.as_ref()];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, binding.as_ref());
        token_interface::set_authority(
            cpi_context,
            spl_token_2022::instruction::AuthorityType::MintTokens,
            Some(new_authority),
        )?;

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

    pub fn submit_data(
        ctx: Context<SubmitData>,
        data_link: String,
        primary_category: String,
        secondary_category: String,
    ) -> Result<()> {
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
        primary_category_bytes[0..primary_category_string_bytes.len()]
            .copy_from_slice(primary_category_string_bytes);

        let secondary_category_string_bytes = secondary_category.as_bytes();
        if secondary_category_string_bytes.len() > SECONDARY_CATEGORY_SIZE {
            return Err(error!(SyncError::ErrSecondaryCategoryTooLarge));
        }
        let mut secondary_category_bytes = [0u8; SECONDARY_CATEGORY_SIZE];
        secondary_category_bytes[0..secondary_category_string_bytes.len()]
            .copy_from_slice(secondary_category_string_bytes);

        *ctx.accounts.data_submission = Datasubmission {
            version: CURRENT_VERSION,
            data_link: storage_link_bytes,
            agent_response: None,
            data_header: DataHeader {
                primary_category: primary_category_bytes,
                secondary_category: secondary_category_bytes,
                reserved: [0u8; DATAHEADER_RESERVED_SIZE],
            },
            user_id: ctx.accounts.signer.key.clone(),
        };

        if ctx.accounts.user_config.version != CURRENT_VERSION {
            *ctx.accounts.user_config = UserConfig {
                version: CURRENT_VERSION,
                accumulated_credits: 0,
                reserved: [0u8; USER_CONFIG_RESERVED_SIZE],
            };
        }

        Ok(())
    }

    pub fn rate_data(
        ctx: Context<RateData>,
        data_link: String,
        passed: bool,
        rating: u8,
    ) -> Result<()> {
        if !ctx.accounts.agent_config.is_enabled {
            return Err(error!(SyncError::ErrAgentIsNotEnabled));
        }

        if ctx.accounts.data_submission.agent_response.is_some() {
            return Err(error!(SyncError::ErrDataAlreadyRated));
        }

        if rating > 100 {
            return Err(error!(SyncError::ErrInvalidRating));
        }

        let calculated_credits = calculate_credits(rating);

        ctx.accounts.data_submission.agent_response = Some(AgentResponse {
            agent_key: ctx.accounts.signer.key.clone(),
            response: passed,
            rating,
            calculated_credits,
        });

        ctx.accounts.user_config.accumulated_credits += calculated_credits;
        Ok(())
    }

    pub fn claim_credits(ctx: Context<ClaimCredits>) -> Result<()> {
        let accumulated_credits = ctx.accounts.user_config.accumulated_credits;

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.program_state.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let signer_seeds = [
            b"sync_program".as_ref(),
            b"global_state".as_ref(),
            &[ctx.bumps.program_state],
        ];
        let binding = [signer_seeds.as_ref()];
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(binding.as_ref());
        token_interface::mint_to(
            cpi_context,
            (accumulated_credits as u64) * 10u64.pow(ctx.accounts.mint.decimals as u32),
        )?;

        ctx.accounts.user_config.accumulated_credits = 0;

        Ok(())
    }

    pub fn demo_claim_credits(
        ctx: Context<DemoClaimCredits>,
        accumulated_credits: u64,
    ) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.program_state.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let signer_seeds = [
            b"sync_program".as_ref(),
            b"global_state".as_ref(),
            &[ctx.bumps.program_state],
        ];
        let binding = [signer_seeds.as_ref()];
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(binding.as_ref());
        token_interface::mint_to(
            cpi_context,
            (accumulated_credits as u64) * 10u64.pow(ctx.accounts.mint.decimals as u32),
        )?;

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
pub struct TransferMintAuthority<'info> {
    #[account(
        seeds = [b"sync_program".as_ref(), b"global_state".as_ref()],
        bump,
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
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
    #[account(
        init_if_needed,
        payer = signer,
        seeds = [b"sync_program".as_ref(), b"user_config".as_ref(), signer.key().as_ref()],
        bump,
        space = 8 + UserConfig::MAX_SIZE
    )]
    pub user_config: Account<'info, UserConfig>,
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
    #[account(
        mut,
        seeds = [b"sync_program".as_ref(), b"user_config".as_ref(), data_submission.user_id.as_ref()],
        bump,
    )]
    pub user_config: Account<'info, UserConfig>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimCredits<'info> {
    #[account(
        seeds = [b"sync_program".as_ref(), b"global_state".as_ref()],
        bump,
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(
        mut,
        seeds = [b"sync_program".as_ref(), b"user_config".as_ref(), signer.key().as_ref()],
        bump,
    )]
    pub user_config: Account<'info, UserConfig>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct DemoClaimCredits<'info> {
    #[account(
        seeds = [b"sync_program".as_ref(), b"global_state".as_ref()],
        bump,
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(
        mut,
        seeds = [b"sync_program".as_ref(), b"user_config".as_ref(), signer.key().as_ref()],
        bump,
    )]
    pub user_config: Account<'info, UserConfig>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
