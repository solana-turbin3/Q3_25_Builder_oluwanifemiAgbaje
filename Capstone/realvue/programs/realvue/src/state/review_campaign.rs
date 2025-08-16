use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ReviewCampaign{
    pub merchant: Pubkey, // owner of the campaign
    #[max_len(32)]
    pub name: String, // name of the campaign
    #[max_len(64)]
    pub product_id: String,
    pub deposit_amount: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub active: bool,
    pub reviews_needed: u16,
    pub approved_count: u16,
    pub refunded: bool,
    pub vault_bump: u8,
    pub bump: u8, 
}