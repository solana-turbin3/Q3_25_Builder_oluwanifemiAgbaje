use anchor_lang::prelude::*;

use crate::{errors::PlatformError, PlatformConfig, ReviewAccount, ReviewCampaign, ReviewerAccount};

#[derive(Accounts)]
pub struct ApproveReview<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

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
        seeds = [b"reviewer", reviewer_account.reviewer.key().as_ref()],
        bump = reviewer_account.bump
    )]
    pub reviewer_account: Account<'info, ReviewerAccount>,
    
    #[account(
        mut,
        has_one = admin, // Ensure only the platform admin can approve
        seeds = [b"realvue", platform.seed.to_le_bytes().as_ref(), platform.admin.key().as_ref()],
        bump = platform.bump,
    )]
    pub platform: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> ApproveReview<'info> {
    pub fn approve_review(&mut self, approved: bool, flagged_reason: Option<String>) -> Result<()> {
        
        require!(
            self.platform.is_active,
            PlatformError::PlatformInactive
        );

        require!(
            !self.review_account.approved && self.review_account.flagged_reason.is_empty(),
            PlatformError::ReviewAlreadyApproved
        );

        require!(
            self.campaign.active,
            PlatformError::CampaignNotActive
        );

          let is_valid = self.validate_review_authenticity()?;

        if approved && is_valid {
            self.review_account.approved = true;
            
            // Update campaign approved count
            self.campaign.approved_count = self.campaign.approved_count
                .checked_add(1)
                .ok_or(PlatformError::ArithmeticOverflow)?;
            
            // Update reviewer stats
            self.reviewer_account.approved_count = self.reviewer_account.approved_count
                .checked_add(1)
                .ok_or(PlatformError::ArithmeticOverflow)?;
            
            // Update reviewer rank based on approved count
            self.update_reviewer_rank()?;
            
            // Update platform total reviews
            self.platform.total_reviews = self.platform.total_reviews
                .checked_add(1)
                .ok_or(PlatformError::ArithmeticOverflow)?;
                
        } else {
            // FLAG PATH
            self.review_account.approved = false;

            self.review_account.flagged_reason = flagged_reason
                .unwrap_or_else(|| "Review verification failed".to_string());
                
            require!(
                self.review_account.flagged_reason.len() <= 64,
                PlatformError::ReasonTooLong
            );
        }

        Ok(())
    }

   pub fn validate_review_authenticity(&self) -> Result<bool> {
       
        if self.review_account.tx_id.is_empty() {
            return Ok(false);
        }

        // for future reference
        // if self.review_account.tx_id.len() < 88 { 
        //     return Ok(false);
        // }

        // Reviewer should not be the merchant
        if self.review_account.reviewer == self.campaign.merchant {
            return Ok(false);
        }

          Ok(true)
    }

   pub fn update_reviewer_rank(&mut self) -> Result<()> {

        let approved_count = self.reviewer_account.approved_count;
        
        let new_rank = match approved_count {
            0..=4 => 1,      // Bronze
            5..=14 => 2,     // Silver  
            15..=49 => 3,    // Gold
            50..=99 => 4,    // Platinum
            _ => 5,          // Diamond
        };

        self.reviewer_account.rank = new_rank;
        
        Ok(())
    }
}