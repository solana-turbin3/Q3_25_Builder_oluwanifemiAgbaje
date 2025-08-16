use anchor_lang::prelude::*;

use crate::{errors::PlatformError, PlatformConfig, ReviewAccount, ReviewCampaign, ReviewerAccount};

#[derive(Accounts)]
pub struct MakeReview<'info> {
    #[account(mut)]
    pub reviewer: Signer<'info>,

    #[account(
        init,
        payer = reviewer,
        seeds = [campaign.key().as_ref(), reviewer.key().as_ref()],
        bump,
        space = 8 + ReviewAccount::INIT_SPACE
    )]
    pub review_account: Account<'info, ReviewAccount>,

    #[account(
        mut,
        seeds = [b"campaign", campaign.name.as_str().as_bytes(), campaign.merchant.key().as_ref()],
        bump= campaign.bump,
    )]
    pub campaign: Account<'info, ReviewCampaign>,

    #[account(
        mut,
        seeds = [b"reviewer", reviewer_account.reviewer.key().as_ref()],
        bump = reviewer_account.bump
    )]
    pub reviewer_account: Account<'info, ReviewerAccount>,
    
    #[account(
        mut,
        seeds = [b"realvue", platform.seed.to_le_bytes().as_ref(), platform.admin.key().as_ref()],
        bump = platform.bump,
    )]
    pub platform: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> MakeReview<'info> {
    pub fn make_review(
        &mut self,
        description: String,
        tx_id: String,
        bumps: &MakeReviewBumps,
    ) -> Result<()> {

        let current_time = Clock::get()?.unix_timestamp as i64;

         require!(
            self.campaign.active,
            PlatformError::CampaignNotActive
        );

        require!(
            current_time >= self.campaign.start_time,
            PlatformError::CampaignNotStarted
        );

        require!(
            current_time <= self.campaign.end_time,
            PlatformError::CampaignEnded
        );

        require!(
            !description.is_empty(),
            PlatformError::EmptyReviewDescription
        );

        require!(
            description.len() <= 500,
            PlatformError::ReviewDescriptionTooLong
        );

        require!(
            !tx_id.is_empty(),
            PlatformError::InvalidTransactionId
        );

        self.review_account.set_inner(ReviewAccount {
            campaign_id: self.campaign.key(),
            reviewer: self.reviewer.key(),
            description,
            approved: false,
            reward_claimed: false,
            flagged_reason: "".to_string(),
            tx_id,
            reviewer_rank: self.reviewer_account.rank,
            timestamp: current_time,
            bump: bumps.review_account,
        });

        self.platform.total_reviews = self.platform.total_reviews
            .checked_add(1)
            .ok_or(PlatformError::ArithmeticOverflow)?;

        Ok(())
    }
}
