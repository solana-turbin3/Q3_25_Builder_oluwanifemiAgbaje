use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PlatformConfig{
    pub seed: u64,
    pub admin: Pubkey, //admin of the platform
    pub rev_mint: Pubkey, // rev token mint
    pub platform_fee: u16,
    pub is_active: bool,
    pub total_campaigns: u64,
    pub total_reviews: u64,
    pub total_fees_collected: u64,
    pub rev_bump: u8,
    pub treasury_bump: u8,
    pub bump: u8,
}