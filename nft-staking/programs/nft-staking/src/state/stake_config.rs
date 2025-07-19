use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeConfig{
    pub points_per_stake: u8, // points earned per stake
    pub max_stake: u8,  //max stake allowed
    pub freeze_period: u32, // how long user want to stake
    pub rewards_bump: u8, 
    pub points_to_rewards_multiplier: u32,
    pub bump: u8,
}