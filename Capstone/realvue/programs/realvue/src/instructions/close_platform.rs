use anchor_lang::prelude::*;

use crate::{errors::PlatformError,  state::PlatformConfig};

#[derive(Accounts)]
pub struct ClosePlatform<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"realvue", platform.seed.to_le_bytes().as_ref(), platform.admin.key().as_ref()],
        bump = platform.bump,
        close = admin,
    )]
    pub platform: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> ClosePlatform<'info> {
    pub fn close_platform(
        &mut self,
    ) -> Result<()> {
        require!(self.platform.is_active, PlatformError::PlatformInactive);
        self.platform.is_active = false;

        Ok(())
    }
}
