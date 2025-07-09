#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

// Program ID, on chaIn addreess that program lives on
declare_id!("5MVhNfqheWeTeMncshZW3A6XTxnsMFbshT2XmieXBVHH");  


#[program] // module that contains all the instructions
pub mod vault {
    use super::*;

    // Instructions
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,     // A system account only stores lamport and it is controlled by the system program

    #[account(
        init,
        payer = signer,
        seeds = [b"state", signer.key().as_ref()], //
        bump,
        space = 8 + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System> //System program, needed for transfers and account creation
}

impl <'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        // Rent exempt, the minimum balance that an account needs to become active or initialized
        let rent_exempt: u64 = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());

        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer{
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_account); //Used when user is signing the transaction

        transfer(cpi_ctx, rent_exempt)?;

        // Store the bump seeds in the state account for future use
        // These bumps are needed to recreate the PDA addresses later
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;
        Ok(())

    }
}

#[derive(Accounts)]
pub struct Deposit<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut, // value of lamports changes
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>
}

impl <'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        
        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer{
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);

        transfer(cpi_ctx, amount)?;

        Ok(())

    }
}

#[derive(Accounts)]
pub struct Withdraw<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>, // vault_state is not mutable because it just storing the bumps

    pub system_program: Program<'info, System>
}

impl <'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        // check that the withdraw leaves the vault with a rent-exempt balance
        let vault_info = self.vault.to_account_info();

        // Get the rent-exempt minimum for the vault account
        let rent_exempt_balance = Rent::get()?.minimum_balance(vault_info.data_len());

        // Check vault has enough lamports to satisfy the rent-exempt requirement after withdrawal
        let remaining_balance = vault_info.lamports()
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;

        require!(
            remaining_balance >= rent_exempt_balance,
            ErrorCode::ViolateRentExemption
        );

        // check vault has enough for this withdrawal minus rent exempt
        require!(
            vault_info.lamports() >= amount,
            ErrorCode::InsufficientFunds
        );

        // check that the account has enough funds for the user to withdraw
        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer{
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info(),
        };

        let pda_signer_seeds=[
            b"vault", 
            self.vault_state.to_account_info().key.as_ref(),
           &[self.vault_state.vault_bump],
            ];
        let seeds = [&pda_signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, &seeds); //Used when a PDA is signing the transaction i.e the vault is a pda 

        transfer(cpi_ctx, amount)?;

        Ok(())

    }
}

#[derive(Accounts)]
pub struct Close<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump,
        close = signer,
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>
}

impl <'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer{
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info(),
        };

        let pda_signer_seeds=[
            b"vault", 
            self.vault_state.to_account_info().key.as_ref(),
           &[self.vault_state.vault_bump],
            ];
        let seeds = [&pda_signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, &seeds);

        transfer(cpi_ctx, self.vault.lamports())?;

        Ok(())

    }
}

#[account]
#[derive(InitSpace)] // this macro does not take into consideration the anchor discriminator
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8
}

#[error_code]
pub enum ErrorCode {
    #[msg("Vault does not have enough lamports to stay rent-exempt after withdrawal.")]
    ViolateRentExemption,
    
    #[msg("Vault does not have enough funds to fulfill this withdrawal.")]
    InsufficientFunds,

    #[msg("Arithmetic underflow occurred.")]
    Underflow,
}


// impl Space for VaultState {
//     const INIT_SPACE: usize = 1 + 1;
// }
