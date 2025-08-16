use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::{
    token_interface::{Mint, TokenInterface},
};

use crate::{errors::PlatformError, state::PlatformConfig};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct InitializePlatform<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"realvue", seed.to_le_bytes().as_ref(), admin.key().as_ref()],
        bump,
        space = 8 + PlatformConfig::INIT_SPACE,
    )]
    pub platform: Account<'info, PlatformConfig>,

    #[account(
        init,
        payer = admin,
        seeds = [b"rev", platform.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = platform,
        mint::freeze_authority = platform
    )]
    pub rev_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"treasury", platform.key().as_ref()],
        bump
    )]
    pub treasury: SystemAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializePlatform<'info> {
    pub fn init_platform(
        &mut self,
        seed: u64,
        platform_fee: u16,
        bumps: &InitializePlatformBumps,
    ) -> Result<()> {
        
        // Validate platform fee (max 10% = 1000 basis points)
        require!(platform_fee <= 1000, PlatformError::InvalidFeePercentage);

        self.platform.set_inner(PlatformConfig {
            seed,
            admin: self.admin.key(),
            rev_mint: self.rev_mint.key(),
            platform_fee,
            is_active: true,
            total_campaigns: 0,
            total_reviews: 0,
            total_fees_collected: 0,
            rev_bump: bumps.rev_mint,
            treasury_bump: bumps.treasury,
            bump: bumps.platform,
        });

        // Rent exempt, the minimum balance that an account needs to become active or initialized
        let rent_exempt: u64 = Rent::get()?.minimum_balance(self.treasury.to_account_info().data_len());

        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer{
            from: self.admin.to_account_info(),
            to: self.treasury.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_account); //Used when user is signing the transaction

        transfer(cpi_ctx, rent_exempt)?;

        Ok(())
    }
}
