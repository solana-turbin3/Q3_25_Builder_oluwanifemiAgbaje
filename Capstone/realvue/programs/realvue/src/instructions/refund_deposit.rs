use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::{
    errors::PlatformError,
    state::{PlatformConfig, ReviewCampaign},
};

#[derive(Accounts)]
pub struct RefundDeposit<'info> {
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
        seeds = [b"campaign_vault", campaign.key().as_ref()],
        bump = campaign.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"realvue", platform.seed.to_le_bytes().as_ref(), platform.admin.key().as_ref()],
        bump = platform.bump,
    )]
    pub platform: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> RefundDeposit<'info> {
    pub fn refund_deposit(&mut self) -> Result<()> {
        // Check if campaign merchant is authorized
        require!(
            self.merchant.key() == self.campaign.merchant,
            PlatformError::UnauthorizedMerchant
        );

        // Check if campaign is not active (active: false)
        require!(self.campaign.active, PlatformError::CampaignNotActive);

        // Check if approved_count < reviews_needed
        require!(
            self.campaign.approved_count < self.campaign.reviews_needed,
            PlatformError::CampaignTargetMet
        );

        let vault_balance = self.vault.lamports();

        // Calculate refundable amount
        let refundable_amount = self.calculate_refundable_amount(vault_balance)?;

        require!(refundable_amount > 0, PlatformError::NoRefundAvailable);

        // Transfer refund to merchant
        self.transfer_refund_to_merchant(refundable_amount)?;

        // Mark campaign as refunded
        self.campaign.refunded = true;

        self.campaign.active = false;

        Ok(())
    }

    pub fn calculate_refundable_amount(&self, vault_balance: u64) -> Result<u64> {

        // If no reviews were approved, refund everything except platform fee
        if self.campaign.approved_count == 0 {
            let fee_amount = (self.campaign.deposit_amount as u128)
                .checked_mul(self.platform.platform_fee as u128)
                .unwrap()
                .checked_div(10000)
                .unwrap() as u64;

            let expected_refund = self
                .campaign
                .deposit_amount
                .checked_sub(fee_amount)
                .ok_or(PlatformError::ArithmeticOverflow)?;

            return Ok(std::cmp::min(expected_refund, vault_balance));
        }

        // If some reviews were approved, return vault balance
        Ok(vault_balance)
    }

    pub fn transfer_refund_to_merchant(&self, amount: u64) -> Result<()> {
        let campaign_key = self.campaign.key();
        let seeds = &[
            &b"campaign_vault"[..],
            &campaign_key.as_ref(),
            &[self.campaign.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.merchant.to_account_info(),
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
