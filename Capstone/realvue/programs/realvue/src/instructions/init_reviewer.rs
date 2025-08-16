use anchor_lang::prelude::*;

use crate::ReviewerAccount;

#[derive(Accounts)]
pub struct InitializeReviewer<'info> {
    #[account(mut)]
    pub reviewer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = reviewer,
        seeds = [b"reviewer", reviewer.key().as_ref()],
        space = 8 + ReviewerAccount::INIT_SPACE,
        bump
    )]
    pub reviewer_account: Account<'info, ReviewerAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeReviewer<'info> {
    pub fn init_reviewer(&mut self, bumps: &InitializeReviewerBumps) -> Result<()> {

        self.reviewer_account.set_inner(ReviewerAccount {
            reviewer: self.reviewer.key(),
            approved_count: 0,
            rank: 1,
            total_earned: 0,
            bump: bumps.reviewer_account,
        });

        Ok(())
    }
}
