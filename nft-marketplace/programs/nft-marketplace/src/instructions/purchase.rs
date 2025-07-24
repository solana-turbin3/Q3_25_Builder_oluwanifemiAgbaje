use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, TransferChecked, Mint, TokenAccount, TokenInterface}
};

use crate::{errors::MarketplaceError, state::{Listing, Marketplace}};


#[derive(Accounts)]
pub struct Purchase<'info>{
    #[account(mut)]
    pub buyer: Signer<'info>, // buyer of the nft

    /// CHECK: This is the seller's wallet and will receive SOL. It's safe because we validate it against `listing.seller`
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    pub seller_mint: InterfaceAccount<'info, Mint>, // the mint of the nft

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = seller_mint,
        associated_token::authority = buyer
    )]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>, // ata to recieve the nft
    
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump
    )]
    pub treasury: SystemAccount<'info>, // just storing marketplace fees 

    #[account(
        mut,
        close = seller,
        seeds = [marketplace.key().as_ref(), seller_mint.key().as_ref()],
        bump = listing.bump,
    )]
    pub listing: Account<'info, Listing>,

    #[account(
        mut,
        associated_token::mint = seller_mint,
        associated_token::authority = listing
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>, // ata holding the nft owned by the listing PDA

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface >,
    pub system_program: Program<'info, System>,
}

impl<'info> Purchase<'info>{
    pub fn transfer_nft(&mut self,) -> Result<()>{
        // Validate listing and seller
        require!(self.listing.is_active && self.listing.seller == self.seller.key(), MarketplaceError::ListingNotActive);

        let marketplace_key = self.marketplace.key();
        let seller_mint_key = self.seller_mint.key();
        let seeds  = &[
            marketplace_key.as_ref(),
            seller_mint_key.as_ref(),
            &[self.listing.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked{
            from: self.vault.to_account_info(),
            mint: self.seller_mint.to_account_info(),
            to: self.buyer_ata.to_account_info(),
            authority: self.listing.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_ctx, 1, self.seller_mint.decimals)

    }

    pub fn transfer_sol(&mut self) -> Result<()>{

        // Calculate marketplace fee (percentage of listing price)
        let fee_lamports = (self.marketplace.fee as u64)
            .checked_mul(self.listing.price)
            .ok_or(MarketplaceError::MathOverflow)?
            .checked_div(100)
            .ok_or(MarketplaceError::MathOverflow)?;
    
        //calculate seller fee - marketplace fee in lamports
        let seller_fee = self.listing.price
            .checked_sub(fee_lamports)
            .ok_or(MarketplaceError::MathOverflow)?;

        //transfer marketplace fee to treasury
        let treasury_transfer_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.buyer.to_account_info(),
                to: self.treasury.to_account_info(),
            },
        );
        transfer(treasury_transfer_ctx, fee_lamports)?;

        // transfer lamports to seller
        let seller_transfer_ctx = CpiContext::new(
            self.token_program.to_account_info(),
             Transfer{
                from: self.buyer.to_account_info(),
                to: self.seller.to_account_info()
             },
        );
        transfer(seller_transfer_ctx, seller_fee)
    }

}