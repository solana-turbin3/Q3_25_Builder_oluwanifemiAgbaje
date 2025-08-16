use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::{associated_token::{AssociatedToken}, token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface}};

use crate::{errors::PlatformError,  state::{PlatformConfig, ReviewCampaign}};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,

    #[account(
        init, 
        payer = merchant,
        seeds = [b"campaign", name.as_str().as_bytes(), merchant.key().as_ref()],
        bump,
        space = 8 + ReviewCampaign::INIT_SPACE
    )]
    pub campaign: Account<'info, ReviewCampaign>,

    #[account(
        init_if_needed,
        payer = merchant,
        associated_token::mint= rev_mint,
        associated_token::authority= merchant 
    )]
    pub merchant_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        has_one = rev_mint,
        seeds = [b"realvue", platform.seed.to_le_bytes().as_ref(), platform.admin.key().as_ref()],
        bump = platform.bump,
    )]
    pub platform: Account<'info, PlatformConfig>,

    #[account(
        mut,
        seeds = [b"rev", platform.key().as_ref()],
        bump= platform.rev_bump,
        mint::decimals = 6,
        mint::authority = platform,
        mint::freeze_authority = platform
    )]
    pub rev_mint: InterfaceAccount<'info, Mint>,

    
    #[account(
        mut,
        seeds = [b"campaign_vault", campaign.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"treasury", platform.key().as_ref()],
        bump = platform.treasury_bump
    )]
    pub treasury: SystemAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateCampaign<'info>{
    pub fn create_campaign(&mut self, name: String, product_id: String,
    deposit_amount: u64, start_time: i64, end_time: i64, reviews_needed: u16, bumps: &CreateCampaignBumps) -> Result<()>{

        const MIN_DEPOSIT: u64 = 100_000_000;
        require!(
            // Minimum deposit check ( 0.1 SOL = 100,000,000 lamports)
            deposit_amount >= MIN_DEPOSIT,
            PlatformError::InsufficientDepositAmount
        );


        self.campaign.set_inner(ReviewCampaign {
            merchant: self.merchant.key(),
            name,
            product_id, 
            deposit_amount, 
            start_time, 
            end_time, 
            active: true, 
            reviews_needed, 
            approved_count: 0,
            refunded: false, 
            vault_bump: bumps.vault,
            bump: bumps.campaign
            });

        let rent_exempt: u64 = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());

        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer{
            from: self.merchant.to_account_info(),
            to: self.vault.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_account); //Used when user is signing the transaction

        transfer(cpi_ctx, rent_exempt)?;
        

            let cpi_program = self.system_program.to_account_info();
            
            let cpi_accounts = Transfer{
                from: self.merchant.to_account_info(),
                to: self.vault.to_account_info(),
            };

            let ctx = CpiContext::new(cpi_program, cpi_accounts);

            transfer(ctx, deposit_amount)?;

            // Calculate fee amount based on fee percentage (basis points)
            let fee_amount = (deposit_amount as u128)
                .checked_mul(self.platform.platform_fee as u128)
                .unwrap()
                .checked_div(10000)
                .unwrap() as u64;

            // Mint REV tokens equal to deposited SOL (1:1 ratio)
            let rev_amount = deposit_amount;

            self.mint_rev_token(rev_amount)?;

            self.deduct_fee(fee_amount)?;

            // Update platform statistics
        self.platform.total_campaigns = self.platform.total_campaigns
            .checked_add(1)
            .ok_or(PlatformError::ArithmeticOverflow)?;

        self.platform.total_fees_collected = self.platform.total_fees_collected
            .checked_add(fee_amount)
            .ok_or(PlatformError::ArithmeticOverflow)?;
            
        Ok(())
    }

    pub fn mint_rev_token(&self, amount: u64) -> Result<()>{
         let cpi_program = self.token_program.to_account_info();

         let admin_key = self.platform.admin.key();
         let seed = self.platform.seed.to_le_bytes();
         let seeds = &[
            &b"realvue"[..],
            &seed.as_ref(),
            &admin_key.as_ref(),
            &[self.platform.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_accounts= MintTo{
            mint: self.rev_mint.to_account_info(),
            to: self.merchant_ata.to_account_info(),
            authority: self.platform.to_account_info(),
        };

        let ctx= CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        mint_to(ctx, amount)?;

        Ok(())
    }
    pub fn deduct_fee(&self, amount: u64) -> Result<()>{

        let vault_balance = self.vault.lamports();
    
        require!(
            vault_balance >= amount,
            PlatformError::InsufficientVaultFunds
        );
        let cpi_program = self.system_program.to_account_info();

        let campaign_key = self.campaign.key();
        let seeds = &[
            &b"campaign_vault"[..],
            &campaign_key.as_ref(),
            &[self.campaign.vault_bump], 
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_accounts= Transfer{
            from: self.vault.to_account_info(),
            to: self.treasury.to_account_info(),
        };

        let ctx= CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(ctx, amount)?;

        Ok(())
    }
}