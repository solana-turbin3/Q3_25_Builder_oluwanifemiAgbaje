use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{MasterEditionAccount, Metadata, MetadataAccount},
    token_interface::{transfer_checked, TransferChecked, Mint, TokenAccount, TokenInterface}
};

use crate::state::{Listing, Marketplace};


#[derive(Accounts)]
pub struct List<'info>{
    #[account(mut)]
    pub seller: Signer<'info>,
    
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,
    pub seller_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = seller_mint,
        associated_token::authority = seller
    )]
    pub seller_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = seller,
        seeds = [marketplace.key().as_ref(), seller_mint.key().as_ref()],
        bump,
        space = 8 + Listing::INIT_SPACE
    )]
    pub listing: Account<'info, Listing>,

    #[account(
        init, 
        payer = seller,
        associated_token::mint = seller_mint,
        associated_token::authority = listing
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub collection_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            seller_mint.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),
        bump,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
        constraint = metadata.collection.as_ref().unwrap().verified == true
    )]
    pub metadata: Account<'info, MetadataAccount>,

    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            seller_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub master_edition: Account<'info, MasterEditionAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface >,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>
}

impl<'info>  List<'info> {
    pub fn  create_listing(&mut self, price: u64, bumps: ListBumps) -> Result<()>{
        self.listing.set_inner(Listing { 
            seller: self.seller.key(),
            seller_mint: self.seller_mint.key(),
            price,
            is_active: true,
            bump: bumps.listing
         });
        Ok(())
    
    }

    pub fn deposit_nft(&mut self,) -> Result<()>{
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked{
            from: self.seller_ata.to_account_info(),
            mint: self.seller_mint.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.seller.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, self.seller_ata.amount, self.seller_mint.decimals)?;


        Ok(())
    }
}