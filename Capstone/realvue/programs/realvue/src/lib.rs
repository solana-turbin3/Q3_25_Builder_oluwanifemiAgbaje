#![allow(unexpected_cfgs, deprecated)]

use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;

declare_id!("CB9cLPfpZM2Dkjrep4LhiNXCpFa5iXhU3Jjr7TDFR8XF");

#[program]
pub mod realvue {
    use super::*;

    pub fn init_platform(ctx: Context<InitializePlatform>, seed:u64, platform_fee: u16) -> Result<()> {
        ctx.accounts.init_platform(seed, platform_fee, &ctx.bumps)?;
        Ok(())
    }
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        name: String,
        product_id: String,
        deposit_amount: u64,
        start_time: i64,
        end_time: i64,
        reviews_needed: u16,
    ) -> Result<()> {
        ctx.accounts.create_campaign(
            name,
            product_id,
            deposit_amount,
            start_time,
            end_time,
            reviews_needed,
            &ctx.bumps,
        )?;
        Ok(())
    }

    pub fn init_reviewer(ctx: Context<InitializeReviewer>) -> Result<()> {
        ctx.accounts.init_reviewer(&ctx.bumps)?;
        Ok(())
    }

    pub fn make_review(ctx: Context<MakeReview>, description: String, tx_id: String) -> Result<()> {
        ctx.accounts.make_review(description, tx_id, &ctx.bumps)?;
        Ok(())
    }

    pub fn approve_review(
        ctx: Context<ApproveReview>,
        approved: bool,
        flagged_reason: Option<String>,
    ) -> Result<()> {
        ctx.accounts.approve_review(approved, flagged_reason)?;
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        ctx.accounts.claim_reward()?;
        Ok(())
    }

    pub fn claim_fee(ctx: Context<ClaimFee>) -> Result<()> {
        ctx.accounts.claim_fee()?;
        Ok(())
    }

    pub fn close_campaign(ctx: Context<CloseCampaign>) -> Result<()> {
        ctx.accounts.close_campaign()?;
        Ok(())
    }

    pub fn refund_deposit(ctx: Context<RefundDeposit>) -> Result<()> {
        ctx.accounts.refund_deposit()?;
        Ok(())
    }

    pub fn close_reviewer(ctx: Context<CloseReviewer>) -> Result<()> {
        ctx.accounts.close_reviewer()?;
        Ok(())
    }

    pub fn close_platform(ctx: Context<ClosePlatform>) -> Result<()> {
        ctx.accounts.close_platform()?;
        Ok(())
    }
}
