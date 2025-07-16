use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{ transfer, Mint, Token, TokenAccount, Transfer}};
// use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, Config};

#[derive(Accounts)]
pub struct Swap<'info> {
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
    pub user_x: Account<'info, TokenAccount>, // user's associated token account for x token for swap

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user
    )]
    pub user_y: Account<'info, TokenAccount>, // user's associated token account for y token for swap


    pub token_program: Program<'info, Token>, //token program
    pub associated_token_program: Program<'info, AssociatedToken>, // associated token program
    pub system_program: Program<'info, System>, // system program, we must always call this because it helps transfer SOL
}

impl<'info> Swap<'info> {
    pub fn swap(
        &mut self, 
        swap_x: bool,        // true = swap X for Y, false = swap Y for X
        amount_in: u64,      // amount of tokens user wants to swap
        min_amount_out: u64, // minimum amount user expects to receive (slippage protection)
    ) -> Result<()> {
        // Safety checks
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount_in > 0, AmmError::InvalidAmount);
        require!(self.vault_x.amount > 0 && self.vault_y.amount > 0, AmmError::NoLiquidityInPool);

        // Calculate swap amounts
        let (amount_out, fee_amount) = self.calculate_swap_amounts(swap_x, amount_in)?;

        // Slippage protection
        require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

        // Perform the swap
        if swap_x {
            // Swap X for Y: User gives X tokens, receives Y tokens
            self.transfer_tokens_to_vault(true, amount_in)?;     // User → Vault (X tokens)
            self.transfer_tokens_to_user(false, amount_out)?;    // Vault → User (Y tokens)
        } else {
            // Swap Y for X: User gives Y tokens, receives X tokens
            self.transfer_tokens_to_vault(false, amount_in)?;    // User → Vault (Y tokens)
            self.transfer_tokens_to_user(true, amount_out)?;     // Vault → User (X tokens)
        }

        // Emit event for tracking
        emit!(SwapEvent {
            user: self.user.key(),
            swap_x,
            amount_in,
            amount_out,
            fee_amount,
        });

        Ok(())
    }

    fn calculate_swap_amounts(&self, swap_x: bool, amount_in: u64) -> Result<(u64, u64)> {
        // Get current pool reserves
        let (reserve_in, reserve_out) = if swap_x {
            (self.vault_x.amount, self.vault_y.amount)
        } else {
            (self.vault_y.amount, self.vault_x.amount)
        };

        // Ensure we have enough liquidity
        require!(reserve_in > 0 && reserve_out > 0, AmmError::InsufficientBalance);

        // Calculate fee (e.g., 0.3% = 30 basis points)
        let fee_amount = (amount_in as u128)
            .checked_mul(self.config.fee as u128)
            .ok_or(AmmError::Overflow)?
            .checked_div(10000)  // Fee is in basis points (0.3% = 30/10000)
            .ok_or(AmmError::Overflow)? as u64;

        // Amount after fee
        let amount_in_after_fee = amount_in
            .checked_sub(fee_amount)
            .ok_or(AmmError::Overflow)?;

        // Constant Product Formula: x * y = k
        // When adding amount_in_after_fee to reserve_in, we need to calculate new reserve_out
        // such that: (reserve_in + amount_in_after_fee) * new_reserve_out = reserve_in * reserve_out
        // Therefore: new_reserve_out = (reserve_in * reserve_out) / (reserve_in + amount_in_after_fee)
        // Amount out = reserve_out - new_reserve_out

        let new_reserve_in = (reserve_in as u128)
            .checked_add(amount_in_after_fee as u128)
            .ok_or(AmmError::Overflow)?;

        let k = (reserve_in as u128)
            .checked_mul(reserve_out as u128)
            .ok_or(AmmError::Overflow)?;

        let new_reserve_out = k
            .checked_div(new_reserve_in)
            .ok_or(AmmError::DivisionByZero)?;

        let amount_out = (reserve_out as u128)
            .checked_sub(new_reserve_out)
            .ok_or(AmmError::InsufficientBalance)?
            .try_into()
            .map_err(|_| AmmError::Overflow)?;

        // Ensure we don't drain the pool
        require!(amount_out < reserve_out, AmmError::InsufficientBalance);
        require!(amount_out > 0, AmmError::InvalidAmount);

        Ok((amount_out, fee_amount))
    }

    // user deposit amount he wants to swap
    fn transfer_tokens_to_vault(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = if is_x {
            (self.user_x.to_account_info(), self.vault_x.to_account_info())
        } else {
            (self.user_y.to_account_info(), self.vault_y.to_account_info())
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(ctx, amount)
    }

    fn transfer_tokens_to_user(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = if is_x {
            (self.vault_x.to_account_info(), self.user_x.to_account_info())
        } else {
            (self.vault_y.to_account_info(), self.user_y.to_account_info())
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info(),
        };

        // Create signer seeds for the config account
        let seeds = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(ctx, amount)
    }
}

// Event for tracking swaps
#[event]
pub struct SwapEvent {
    pub user: Pubkey,
    pub swap_x: bool,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
}