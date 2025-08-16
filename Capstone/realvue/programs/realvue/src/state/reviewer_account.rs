use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ReviewerAccount{
    pub reviewer: Pubkey,
    pub approved_count: u16,
    pub rank: u8,
    pub total_earned: u64,
    pub bump: u8
}