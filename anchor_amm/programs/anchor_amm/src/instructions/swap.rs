use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{ transfer, Mint, Token, TokenAccount, Transfer}};
use constant_product_curve::{ConstantProduct, LiquidityPair};

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

        let mut curve = ConstantProduct::init(
            self.vault_x.amount, 
            self.vault_y.amount, 
            self.vault_x.amount, 
            self.config.fee, 
           None
        ).map_err(AmmError::from)?;

        let p = match swap_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y
        };

   let res = curve.swap(p, amount_in, min_amount_out).map_err(AmmError::from)?;

        require!(res.deposit != 0, AmmError::InvalidAmount);
        require!(res.withdraw != 0, AmmError::InvalidAmount);

        // Slippage protection
        require!(res.withdraw >= min_amount_out, AmmError::SlippageExceeded);

        // Perform the swap
        self.transfer_tokens_to_vault(swap_x, res.deposit)?;
        self.transfer_tokens_to_user(swap_x, res.withdraw)?;


        //transfer fee
        self.transfer_fee_to_lps(swap_x, res.fee)?;

        // Emit event for tracking
        emit!(SwapEvent {
            user: self.user.key(),
            swap_x,
            amount_in,
            min_amount_out,
            fee_amount: res.fee,
        });
        Ok(())

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

     pub fn transfer_fee_to_lps(&self, swap_x: bool, amount: u64) -> Result<()> {

        let ( from, to ) = match swap_x {
            true => (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info()
            ),
            false => (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info()
            )
        };

        let cpi_accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.user.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        transfer(cpi_ctx, amount)
    }
}

// Event for tracking swaps
#[event]
pub struct SwapEvent {
    pub user: Pubkey,
    pub swap_x: bool,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub fee_amount: u64,
}