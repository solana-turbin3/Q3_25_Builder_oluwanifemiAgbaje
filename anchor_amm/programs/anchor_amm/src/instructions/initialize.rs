use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

use crate::Config;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info>{
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub mint_x: Account<'info, Mint>, // x token
    pub mint_y: Account<'info, Mint>,// y token

    #[account(
        init,
        payer = initializer,
        seeds = [b"lp", config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Account<'info, Mint>, // liquidity provider token

    #[account(
        init,
        payer = initializer,
        seeds = [b"config", seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Config::INIT_SPACE,
    )]
    pub config: Account<'info, Config>, // the escrow service / swap service / liquidity pool

    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Account<'info, TokenAccount>, // mint_x associated token account

    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Account<'info, TokenAccount>, // mint_y associated token account

    pub token_program: Program<'info, Token>, //token program- 
    pub associated_token_program: Program<'info, AssociatedToken>, // associated token program
    pub system_program: Program<'info, System>, // system program, we must always call this because it helps transfer SOL
}

impl<'info> Initialize<'info>{
    pub fn initialize(&mut self, seed: u64, fee: u16, authority: Option<Pubkey>, bumps: InitializeBumps ) -> Result<()>{
        self.config.set_inner(Config { //.set_inner helps auto-fill the parameters without missing one
             seed, 
             authority, 
             mint_x: self.mint_x.key(), 
             mint_y: self.mint_y.key(), 
             fee, 
             locked: false, 
             config_bump: bumps.config, 
             lp_bump: bumps.mint_lp  
            });
        Ok(())
    }
}