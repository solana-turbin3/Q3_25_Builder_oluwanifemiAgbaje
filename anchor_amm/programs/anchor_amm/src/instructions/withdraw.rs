use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer}};
// use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // the liqidity provider that wants to deposit two different tokens into the liquidity pool
    pub mint_x: Account<'info, Mint>, // token x
    pub mint_y: Account<'info, Mint>, // token y

    #[account(
        has_one= mint_x,
        has_one= mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>, //the escrow service / swap service / liquidity pool

    #[account(     
        mut,   // mutable because the supply of the mint will change
        seeds = [b"lp", config.key().as_ref()],
        bump= config.lp_bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Account<'info, Mint>, // the liquidity provider token

    #[account(
        mut, // mutable because lamports change
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Account<'info, TokenAccount>, // mint_x associated token account

    #[account(
        mut, // mutable because lamports change
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Account<'info, TokenAccount>, // mint_y associated token account

    #[account(
        mut, 
        associated_token::mint = mint_x,
        associated_token::authority = user
    )]
    pub user_x: Account<'info, TokenAccount>, // liquidity provider's associated token account for x token

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user
    )]
    pub user_y: Account<'info, TokenAccount>, // liquidity provider's associated token account for y token

    #[account(
        init_if_needed, // we dont know if this is a new liquidity provider so create ata if one does not exist
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user
    )]
    pub user_lp: Account<'info, TokenAccount>, //liquidity provider's associated token account for their lp token requested

    pub token_program: Program<'info, Token>, //token program
    pub associated_token_program: Program<'info, AssociatedToken>, // associated token program
    pub system_program: Program<'info, System>, // system program, we must always call this because it helps transfer SOL
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64, min_x: u64, min_y: u64  ) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked); // check if pool is locked
        require!(amount != 0, AmmError::InvalidAmount); // lp amount requested must not be zero 
         require!(amount <= self.user_lp.amount, AmmError::InvalidAmount);
        require!(self.mint_lp.supply > 0, AmmError::NoLiquidityInPool);

        let (x, y) = self.calculate_withdraw_amounts(amount)?;

        require!(x >= min_x && y >= min_y, AmmError::SlippageExceeded);

        self.transfer_tokens_to_lp( true, x)?;
        self.transfer_tokens_to_lp(false, y)?;

        self.burn_lp_token(amount)

    }
    fn calculate_withdraw_amounts(&self, amount: u64) -> Result<(u64, u64)> {
        // Calculate proportional amounts based on LP token ownership
        // Formula: (vault_amount * lp_amount) / total_lp_supply
        
        let x_amount = (self.vault_x.amount as u128)
            .checked_mul(amount as u128) //// multiplication overflow
            .ok_or(AmmError::Overflow)?
            .checked_div(self.mint_lp.supply as u128) // division overflow
            .ok_or(AmmError::Overflow)? as u64;

        let y_amount = (self.vault_y.amount as u128)
            .checked_mul(amount as u128) // multiplication overflow
            .ok_or(AmmError::Overflow)?
            .checked_div(self.mint_lp.supply as u128) // division overflow
            .ok_or(AmmError::Overflow)? as u64;

        // Ensure we're not trying to withdraw more than what's in the vaults
        require!(x_amount <= self.vault_x.amount, AmmError::InsufficientBalance);
        require!(y_amount <= self.vault_y.amount, AmmError::InsufficientBalance);

        Ok((x_amount, y_amount))
    }

    pub fn transfer_tokens_to_lp(&self, is_x: bool, amount: u64, ) -> Result<()>{
        let (from, to) = match is_x {
            true => (self.vault_x.to_account_info(), self.user_x.to_account_info()), // withdrawal to liquidity provider's x_ata from vault_x
            false => (self.vault_y.to_account_info(), self.user_y.to_account_info()), // withdrawal to liquidity provider's y_ata from vault_y
            };

        let cpi_program = self.token_program.to_account_info(); // invoke the token program 

        let cpi_accounts = Transfer{
            from,
            to,
            authority: self.config.to_account_info()
        };

         let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];

        let signers_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers_seeds);

        transfer(ctx, amount)    }

    pub fn burn_lp_token(&self, amount: u64,) -> Result<()> {

         let cpi_program = self.token_program.to_account_info(); // invoke the token program 

         let cpi_accounts = Burn{
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.user.to_account_info()
         };

         let ctx = CpiContext::new(cpi_program, cpi_accounts);

         burn(ctx, amount)
    }
}