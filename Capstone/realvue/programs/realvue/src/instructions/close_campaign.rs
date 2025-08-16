use anchor_lang::prelude::*;

use crate::{errors::PlatformError, PlatformConfig, ReviewCampaign};

#[derive(Accounts)]
pub struct CloseCampaign<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        mut,
        seeds = [b"campaign", campaign.name.as_str().as_bytes(), campaign.merchant.key().as_ref()],
        bump = campaign.bump,
        close = merchant
    )]
    pub campaign: Account<'info, ReviewCampaign>,

    #[account(
        mut,
        seeds = [b"realvue", platform.seed.to_le_bytes().as_ref(), platform.admin.key().as_ref()],
        bump = platform.bump,
    )]
    pub platform: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> CloseCampaign<'info> {
    pub fn close_campaign(&mut self) -> Result<()> {

        // Check if merchant is authorized
        require!(
            self.merchant.key() == self.campaign.merchant,
            PlatformError::UnauthorizedMerchant
        );

        // Check if campaign is currently active
        require!(
            self.campaign.active,
            PlatformError::CampaignNotActive
        );

        // Check if campaign has reached its end time or target
        // let current_time = Clock::get()?.unix_timestamp;
        // let can_close = current_time >= self.campaign.end_time || 
        //                self.campaign.approved_count >= self.campaign.reviews_needed;

        // require!(
        //     can_close,
        //     PlatformError::CampaignCannotBeClosed
        // );

        // Set campaign as inactive
        self.campaign.active = false;

        Ok(())
    }
}