use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{errors::PlatformError, PlatformConfig};

#[derive(Accounts)]
pub struct ClaimFee<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"realvue", platform.seed.to_le_bytes().as_ref(), platform.admin.key().as_ref()],
        bump = platform.bump,
    )]
    pub platform: Account<'info, PlatformConfig>,

    #[account(
        mut,
        seeds = [b"treasury", platform.key().as_ref()],
        bump = platform.treasury_bump
    )]
    pub treasury: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
impl<'info> ClaimFee<'info> {
    pub fn claim_fee(&mut self) -> Result<()> {
        
        // Manual checks
        require!(
            self.admin.key() == self.platform.admin,
            PlatformError::UnauthorizedAdmin
        );

        let treasury_balance = self.treasury.lamports();

        require!(
            treasury_balance > 0,
            PlatformError::InsufficientTreasuryFunds
        );

        let platform_key = self.platform.key();
        let seeds = &[
            &b"treasury"[..],
            &platform_key.as_ref(),
            &[self.platform.treasury_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.admin.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        transfer(ctx, treasury_balance)?;

        // Update platform statistics with fees claimed
        self.platform.total_fees_collected = self
            .platform
            .total_fees_collected
            .checked_add(treasury_balance)
            .ok_or(PlatformError::ArithmeticOverflow)?;

        Ok(())
    }
}
