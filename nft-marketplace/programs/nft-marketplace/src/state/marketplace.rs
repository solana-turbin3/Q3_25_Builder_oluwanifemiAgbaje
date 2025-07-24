use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Marketplace{
    pub admin: Pubkey,
    pub fee: u16, // basis points
    pub bump: u8,
    pub treasury_bump : u8, // where we keep fees gained in the marketplace 
    pub rewards_bump: u8, // we reward the marketplace users
    #[max_len(32)]
    pub name: String // name of the market place
}