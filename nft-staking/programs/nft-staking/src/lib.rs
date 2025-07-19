#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;

mod state;

mod instructions;
pub mod error;

pub use instructions::*;


declare_id!("FHyfL4nnycvYHDKEo2pjLL3DkdrZwz4EnbHdEPr17Mrb");

#[program]
pub mod nft_staking {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>, points_per_stake: u8, max_stake: u8, freeze_period: u32, points_to_rewards_multiplier:u32) -> Result<()> {
        ctx.accounts.initialize_config(points_per_stake, max_stake, freeze_period, points_to_rewards_multiplier, &ctx.bumps)
    }

    pub fn initialize_user(ctx: Context<InitializeUser>, ) -> Result<()> {
        ctx.accounts.initialize_user(&ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)
    }
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake()
    }

    pub fn claim(ctx: Context<Claim>, points_to_claim: Option<u32>) -> Result<()> {
        ctx.accounts.claim(points_to_claim)
    }
    
}

