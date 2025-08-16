use anchor_lang::prelude::*;

use crate::ReviewerAccount;

#[derive(Accounts)]
pub struct CloseReviewer<'info> {
    #[account(mut)]
    pub reviewer: Signer<'info>,
    #[account(
        mut,
        close = reviewer, 
        seeds = [b"reviewer", reviewer.key().as_ref()],
        bump = reviewer_account.bump
    )]
    pub reviewer_account: Account<'info, ReviewerAccount>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseReviewer<'info> {
    pub fn close_reviewer(&mut self) -> Result<()> {
        Ok(())
    }
}