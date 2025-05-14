use anchor_lang::prelude::*;

declare_id!("3FfiWU89727pbcRppUBpD8ZSMpRxTPBfu324ynPffp2i");

#[program]
pub mod multisig {
    use super::*;

    pub fn create(ctx: Context<Create>, owner1: Pubkey, owner2: Pubkey) -> Result<()> {
        let multisig = &mut ctx.accounts.multisig;
        multisig.owner1 = owner1;
        multisig.owner2 = owner2;
        multisig.confirmed1 = false;
        multisig.confirmed2 = false;
        Ok(())
    }

    pub fn approve(ctx: Context<Approve>) -> Result<()> {
        let multisig = &mut ctx.accounts.multisig;
        let signer = ctx.accounts.signer.key();

        if signer == multisig.owner1 {
            multisig.confirmed1 = true;
        } else if signer == multisig.owner2 {
            multisig.confirmed2 = true;
        } else {
            return Err(error!(ErrorCode::InvalidOwner));
        }

        Ok(())
    }

    pub fn execute(ctx: Context<Execute>) -> Result<()> {
        let multisig = &ctx.accounts.multisig;
        require!(
            multisig.confirmed1 && multisig.confirmed2,
            ErrorCode::NotEnoughApprovals
        );

        msg!("Transaction executed!");
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(owner1: Pubkey, owner2: Pubkey)]
pub struct Create<'info> {
    #[account(
        init,
        seeds = [b"multisig", owner1.as_ref(), owner2.as_ref()],
        bump,
        payer = payer,
        space = 8 + 32*2 + 1*2
    )]
    pub multisig: Account<'info, Multisig>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(mut)]
    pub multisig: Account<'info, Multisig>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct Execute<'info> {
    pub multisig: Account<'info, Multisig>,
}

#[account]
pub struct Multisig {
    pub owner1: Pubkey,
    pub owner2: Pubkey,
    pub confirmed1: bool,
    pub confirmed2: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Not enough approvals")]
    NotEnoughApprovals,
    #[msg("Invalid owner")]
    InvalidOwner,
}