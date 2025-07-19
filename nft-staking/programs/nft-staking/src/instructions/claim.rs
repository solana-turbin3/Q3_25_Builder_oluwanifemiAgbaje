use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, MintTo, Mint, Token, TokenAccount},
};

use crate::{
    error::StakeError,
    state::{ StakeConfig, UserAccount},
};

#[derive(Accounts)]
pub struct Claim<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user
    )]
    pub rewards_ata: Account<'info, TokenAccount>,

    
    #[account(
        seeds = [b"config".as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        mut,
        seeds = [b"reward".as_ref(), config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config
    )]
    pub rewards_mint: Account<'info, Mint>,
    

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info,  UserAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info>Claim<'info> {
    pub fn claim(&mut self, points_to_claim: Option<u32>) -> Result<()>{

        require!(
            self.user_account.points > 0,
            StakeError::NoPointsToClaim
        );

        let points_to_claim = match points_to_claim {
            Some(amount) => {
                require!(
                    amount > 0 && amount <= self.user_account.points,
                    StakeError::InvalidClaimAmount
                );
                amount
            }
            None => self.user_account.points, // Claim all if None
        };

        let reward_amount = (points_to_claim * self.config.points_to_rewards_multiplier) as u64;

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo{
            mint: self.rewards_mint.to_account_info(),
            to: self.rewards_ata.to_account_info(),
            authority: self.config.to_account_info()
        };

        let seeds = &[
            b"config".as_ref(),
            &[self.config.bump]
        ];
        let signers_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers_seeds);

        mint_to(cpi_ctx, reward_amount)?; 

        self.user_account.points -= points_to_claim;
        self.user_account.total_claimed += reward_amount as u32;
        Ok(())
    }
}