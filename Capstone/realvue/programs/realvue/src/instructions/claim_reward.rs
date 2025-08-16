use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

use crate::{errors::PlatformError, PlatformConfig, ReviewAccount, ReviewCampaign, ReviewerAccount};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub reviewer: Signer<'info>,

    #[account(
        mut,
        seeds = [campaign.key().as_ref(), review_account.reviewer.key().as_ref()],
        bump = review_account.bump,
    )]
    pub review_account: Account<'info, ReviewAccount>,

    #[account(
        mut,
        seeds = [b"campaign", campaign.name.as_str().as_bytes(), campaign.merchant.key().as_ref()],
        bump = campaign.bump,
    )]
    pub campaign: Account<'info, ReviewCampaign>,

    #[account(
        mut,
        seeds = [b"campaign_vault", campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"reviewer", reviewer.key().as_ref()],
        bump = reviewer_account.bump
    )]
    pub reviewer_account: Account<'info, ReviewerAccount>,
    
    #[account(
        seeds = [b"realvue", platform.seed.to_le_bytes().as_ref(), platform.admin.key().as_ref()],
        bump = platform.bump,
    )]
    pub platform: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> ClaimReward<'info> {
    pub fn claim_reward(&mut self) -> Result<()> {
        // Manual checks
        require!(
            self.review_account.reviewer == self.reviewer.key(),
            PlatformError::UnauthorizedReviewer
        );

        require!(
            self.review_account.approved == true,
            PlatformError::ReviewNotApproved
        );

        require!(
            self.review_account.reward_claimed == false,
            PlatformError::RewardAlreadyClaimed
        );


        // Calculate reward per reviewer
        let reward_per_reviewer = self.calculate_reward_per_reviewer()?;

        // Check if vault has sufficient balance
        let vault_balance = self.vault.lamports();
        require!(
            vault_balance >= reward_per_reviewer,
            PlatformError::InsufficientVaultFunds
        );

        // Transfer reward to reviewer
        self.transfer_reward_to_reviewer(reward_per_reviewer)?;

        // Mark reward as claimed
        self.review_account.reward_claimed = true;

        // Update reviewer account stats
        self.reviewer_account.total_earned = self.reviewer_account.total_earned
            .checked_add(reward_per_reviewer)
            .ok_or(PlatformError::ArithmeticOverflow)?;

        Ok(())
    }

   pub fn calculate_reward_per_reviewer(&self) -> Result<u64> {
        // Calculate platform fee
        let fee_amount = (self.campaign.deposit_amount as u128)
            .checked_mul(self.platform.platform_fee as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;

        // Amount available for rewards (deposit - platform fee)
        let reward_pool = self.campaign.deposit_amount
            .checked_sub(fee_amount)
            .ok_or(PlatformError::ArithmeticOverflow)?;

        // Reward per reviewer (reward pool / reviews needed)
        let reward_per_reviewer = reward_pool
            .checked_div(self.campaign.reviews_needed as u64)
            .ok_or(PlatformError::RewardCalculationError)?;

        Ok(reward_per_reviewer)
    }

    pub fn transfer_reward_to_reviewer(&self, amount: u64) -> Result<()> {
        let campaign_key = self.campaign.key();
        let seeds = &[
            &b"campaign_vault"[..],
            &campaign_key.as_ref(),
            &[self.campaign.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.reviewer.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        transfer(ctx, amount)?;

        Ok(())
    }
}