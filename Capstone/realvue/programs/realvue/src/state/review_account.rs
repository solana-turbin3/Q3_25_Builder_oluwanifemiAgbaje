use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ReviewAccount{
    pub campaign_id: Pubkey,
    pub reviewer: Pubkey,
    #[max_len(500)]
    pub description: String,
    pub approved: bool,
    pub reward_claimed: bool,
    #[max_len(64)]
    pub flagged_reason: String,
    #[max_len(88)]
    pub tx_id: String, 
    pub reviewer_rank: u8,
    pub timestamp: i64,
    pub bump: u8,
}