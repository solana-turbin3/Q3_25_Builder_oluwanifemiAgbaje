use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Listing{
    pub seller: Pubkey,
    pub seller_mint: Pubkey,
    pub price: u64,
    pub is_active: bool,
    pub bump: u8,
}