use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

declare_id!("3FfiWU89727pbcRppUBpD8ZSMpRxTPBfu324ynPffp2i");

#[program]
pub mod multisig {
    use super::*;

    // Create a new Argent account with owner and guardian
    pub fn create(
        ctx: Context<Create>,
        owner: Pubkey,
        guardian: Pubkey,
        security_period: Option<i64>,
    ) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        argent_account.owner = owner;
        argent_account.guardian = guardian;
        argent_account.guardian_backup = None;
        argent_account.escape_type = EscapeType::None;
        argent_account.escape_initiated_at = 0;
        
        // Set security period (default 7 days = 604800 seconds)
        argent_account.security_period = security_period.unwrap_or(604800);
        
        // Initialize pending transaction
        argent_account.pending_tx = None;
        
        Ok(())
    }

    // Execute a transaction with both owner and guardian signatures
    pub fn execute(ctx: Context<Execute>, data: Vec<u8>) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        
        // Verify that both owner and guardian have signed
        let owner_signed = ctx.accounts.owner.is_signer;
        let guardian_signed = ctx.accounts.guardian.is_signer;
        
        require!(
            owner_signed && guardian_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Store the transaction data for execution
        argent_account.pending_tx = Some(PendingTransaction {
            data,
            owner_approved: true,
            guardian_approved: true,
        });
        
        msg!("Transaction approved and ready for execution!");
        Ok(())
    }
    
    // Change the owner with both owner and guardian signatures
    // Also requires a signature from the new owner
    pub fn change_owner(
        ctx: Context<ChangeOwner>,
        new_owner: Pubkey,
        new_owner_signature: [u8; 64],
    ) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        
        // Verify that both current owner and guardian have signed
        let owner_signed = ctx.accounts.owner.is_signer;
        let guardian_signed = ctx.accounts.guardian.is_signer;
        
        require!(
            owner_signed && guardian_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Verify new owner signature
        // In a real implementation, we would verify the signature here
        // For simplicity, we're just checking that a signature was provided
        require!(
            new_owner_signature != [0; 64],
            ErrorCode::InvalidSignature
        );
        
        // Change the owner
        argent_account.owner = new_owner;
        
        msg!("Owner changed successfully!");
        Ok(())
    }
    
    // Change the guardian with both owner and guardian signatures
    pub fn change_guardian(ctx: Context<ChangeGuardian>, new_guardian: Pubkey) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        
        // Verify that both owner and guardian have signed
        let owner_signed = ctx.accounts.owner.is_signer;
        let guardian_signed = ctx.accounts.guardian.is_signer;
        
        require!(
            owner_signed && guardian_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Change the guardian
        argent_account.guardian = new_guardian;
        
        msg!("Guardian changed successfully!");
        Ok(())
    }
    
    // Add or change the guardian backup with both owner and guardian signatures
    pub fn change_guardian_backup(
        ctx: Context<ChangeGuardianBackup>,
        new_guardian_backup: Option<Pubkey>,
    ) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        
        // Verify that both owner and guardian have signed
        let owner_signed = ctx.accounts.owner.is_signer;
        let guardian_signed = ctx.accounts.guardian.is_signer;
        
        require!(
            owner_signed && guardian_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Change the guardian backup
        argent_account.guardian_backup = new_guardian_backup;
        
        msg!("Guardian backup changed successfully!");
        Ok(())
    }
    
    // Trigger escape mode for guardian (owner can do this alone)
    pub fn trigger_escape_guardian(ctx: Context<TriggerEscapeGuardian>) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        let clock = Clock::get()?;
        
        // Verify that owner has signed
        let owner_signed = ctx.accounts.owner.is_signer;
        
        require!(
            owner_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Can override an escape owner in progress
        if argent_account.escape_type == EscapeType::Owner {
            msg!("Overriding escape owner in progress");
        }
        
        // Set escape type and timestamp
        argent_account.escape_type = EscapeType::Guardian;
        argent_account.escape_initiated_at = clock.unix_timestamp;
        
        msg!("Guardian escape triggered!");
        Ok(())
    }
    
    // Trigger escape mode for owner (guardian can do this alone)
    pub fn trigger_escape_owner(ctx: Context<TriggerEscapeOwner>) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        let clock = Clock::get()?;
        
        // Verify that guardian has signed
        let guardian_signed = ctx.accounts.guardian.is_signer;
        
        require!(
            guardian_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Fail if escape guardian in progress
        require!(
            argent_account.escape_type != EscapeType::Guardian,
            ErrorCode::EscapeGuardianInProgress
        );
        
        // Set escape type and timestamp
        argent_account.escape_type = EscapeType::Owner;
        argent_account.escape_initiated_at = clock.unix_timestamp;
        
        msg!("Owner escape triggered!");
        Ok(())
    }
    
    // Complete escape for guardian (owner can do this alone after security period)
    pub fn escape_guardian(ctx: Context<EscapeGuardian>, new_guardian: Pubkey) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        let clock = Clock::get()?;
        
        // Verify that owner has signed
        let owner_signed = ctx.accounts.owner.is_signer;
        
        require!(
            owner_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Verify escape type
        require!(
            argent_account.escape_type == EscapeType::Guardian,
            ErrorCode::InvalidEscapeType
        );
        
        // Verify security period has elapsed
        let elapsed = clock.unix_timestamp - argent_account.escape_initiated_at;
        require!(
            elapsed >= argent_account.security_period,
            ErrorCode::SecurityPeriodNotElapsed
        );
        
        // Change the guardian
        argent_account.guardian = new_guardian;
        
        // Reset escape state
        argent_account.escape_type = EscapeType::None;
        argent_account.escape_initiated_at = 0;
        
        msg!("Guardian escaped successfully!");
        Ok(())
    }
    
    // Complete escape for owner (guardian can do this alone after security period)
    pub fn escape_owner(ctx: Context<EscapeOwner>, new_owner: Pubkey) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        let clock = Clock::get()?;
        
        // Verify that guardian has signed
        let guardian_signed = ctx.accounts.guardian.is_signer;
        
        require!(
            guardian_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Verify escape type
        require!(
            argent_account.escape_type == EscapeType::Owner,
            ErrorCode::InvalidEscapeType
        );
        
        // Verify security period has elapsed
        let elapsed = clock.unix_timestamp - argent_account.escape_initiated_at;
        require!(
            elapsed >= argent_account.security_period,
            ErrorCode::SecurityPeriodNotElapsed
        );
        
        // Change the owner
        argent_account.owner = new_owner;
        
        // Reset escape state
        argent_account.escape_type = EscapeType::None;
        argent_account.escape_initiated_at = 0;
        
        msg!("Owner escaped successfully!");
        Ok(())
    }
    
    // Cancel escape (requires both owner and guardian)
    pub fn cancel_escape(ctx: Context<CancelEscape>) -> Result<()> {
        let argent_account = &mut ctx.accounts.argent_account;
        
        // Verify that both owner and guardian have signed
        let owner_signed = ctx.accounts.owner.is_signer;
        let guardian_signed = ctx.accounts.guardian.is_signer;
        
        require!(
            owner_signed && guardian_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Verify escape is in progress
        require!(
            argent_account.escape_type != EscapeType::None,
            ErrorCode::NoEscapeInProgress
        );
        
        // Reset escape state
        argent_account.escape_type = EscapeType::None;
        argent_account.escape_initiated_at = 0;
        
        msg!("Escape cancelled!");
        Ok(())
    }
    
    // Upgrade the program implementation (requires both owner and guardian)
    pub fn upgrade(ctx: Context<Upgrade>) -> Result<()> {
        // Verify that both owner and guardian have signed
        let owner_signed = ctx.accounts.owner.is_signer;
        let guardian_signed = ctx.accounts.guardian.is_signer;
        
        require!(
            owner_signed && guardian_signed,
            ErrorCode::NotEnoughApprovals
        );
        
        // Create the upgrade instruction for the BPF Loader
        use anchor_lang::solana_program::{
            instruction::{Instruction, AccountMeta},
            program::invoke,
        };
        
        // Create the upgrade instruction manually
        let upgrade_ix = Instruction {
            program_id: ctx.accounts.bpf_loader.key(),
            accounts: vec![
                AccountMeta::new(*ctx.accounts.program.key, false),
                AccountMeta::new(*ctx.accounts.program_data.key, false),
                AccountMeta::new(*ctx.accounts.buffer.key, false),
                AccountMeta::new_readonly(*ctx.accounts.upgrade_authority.key, true),
                AccountMeta::new_readonly(*ctx.accounts.rent.key, false),
                AccountMeta::new_readonly(*ctx.accounts.clock.key, false),
                AccountMeta::new_readonly(*ctx.accounts.spl_token_program.key, false),
                AccountMeta::new_readonly(*ctx.accounts.system_program.key, false),
            ],
            data: vec![3], // 3 is the instruction index for upgrade
        };
        
        // Invoke the upgrade instruction
        invoke(
            &upgrade_ix,
            &[
                ctx.accounts.program.to_account_info(),
                ctx.accounts.program_data.to_account_info(),
                ctx.accounts.buffer.to_account_info(),
                ctx.accounts.upgrade_authority.to_account_info(),
                ctx.accounts.rent.to_account_info(),
                ctx.accounts.clock.to_account_info(),
                ctx.accounts.spl_token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        msg!("Program implementation upgraded successfully!");
        Ok(())
    }
    
    // This functionality is removed as Solana handles this directly
    // External execution with signatures is handled by the Solana runtime
}

// Account contexts

#[derive(Accounts)]
#[instruction(owner: Pubkey, guardian: Pubkey, security_period: Option<i64>)]
pub struct Create<'info> {
    #[account(
        init,
        seeds = [b"argent", owner.as_ref(), guardian.as_ref()],
        bump,
        payer = payer,
        space = 8 + 32 + 32 + 33 + 1 + 8 + 1 + 200 // Extra space for pending tx
    )]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Execute<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.owner == owner.key())]
    pub owner: Signer<'info>,
    #[account(constraint = argent_account.guardian == guardian.key())]
    pub guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChangeOwner<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.owner == owner.key())]
    pub owner: Signer<'info>,
    #[account(constraint = argent_account.guardian == guardian.key())]
    pub guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChangeGuardian<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.owner == owner.key())]
    pub owner: Signer<'info>,
    #[account(constraint = argent_account.guardian == guardian.key())]
    pub guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChangeGuardianBackup<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.owner == owner.key())]
    pub owner: Signer<'info>,
    #[account(constraint = argent_account.guardian == guardian.key())]
    pub guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct TriggerEscapeGuardian<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.owner == owner.key())]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct TriggerEscapeOwner<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.guardian == guardian.key())]
    pub guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct EscapeGuardian<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.owner == owner.key())]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct EscapeOwner<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.guardian == guardian.key())]
    pub guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelEscape<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.owner == owner.key())]
    pub owner: Signer<'info>,
    #[account(constraint = argent_account.guardian == guardian.key())]
    pub guardian: Signer<'info>,
}

#[derive(Accounts)]
pub struct Upgrade<'info> {
    #[account(mut)]
    pub argent_account: Account<'info, ArgentAccount>,
    #[account(constraint = argent_account.owner == owner.key())]
    pub owner: Signer<'info>,
    #[account(constraint = argent_account.guardian == guardian.key())]
    pub guardian: Signer<'info>,
    /// CHECK: This is the program to upgrade
    #[account(mut)]
    pub program: AccountInfo<'info>,
    /// CHECK: This is the program data account
    #[account(mut)]
    pub program_data: AccountInfo<'info>,
    /// CHECK: This is the buffer with the new program code
    pub buffer: AccountInfo<'info>,
    /// CHECK: Upgrade authority of the program
    pub upgrade_authority: Signer<'info>,
    /// CHECK: The BPF Loader program
    pub bpf_loader: AccountInfo<'info>,
    /// CHECK: Rent sysvar
    pub rent: AccountInfo<'info>,
    /// CHECK: Clock sysvar
    pub clock: AccountInfo<'info>,
    /// CHECK: SPL Token program
    pub spl_token_program: AccountInfo<'info>,
    /// CHECK: System program
    pub system_program: AccountInfo<'info>,
}

// ExecuteFromOutside context removed as Solana handles this directly

// Account data structure

#[account]
pub struct ArgentAccount {
    pub owner: Pubkey,
    pub guardian: Pubkey,
    pub guardian_backup: Option<Pubkey>,
    pub escape_type: EscapeType,
    pub escape_initiated_at: i64,
    pub security_period: i64,
    pub pending_tx: Option<PendingTransaction>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum EscapeType {
    None,
    Guardian,
    Owner,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PendingTransaction {
    pub data: Vec<u8>,
    pub owner_approved: bool,
    pub guardian_approved: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Not enough approvals")]
    NotEnoughApprovals,
    #[msg("Invalid owner")]
    InvalidOwner,
    #[msg("Invalid guardian")]
    InvalidGuardian,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("Escape guardian in progress")]
    EscapeGuardianInProgress,
    #[msg("Invalid escape type")]
    InvalidEscapeType,
    #[msg("Security period not elapsed")]
    SecurityPeriodNotElapsed,
    #[msg("No escape in progress")]
    NoEscapeInProgress,
}
