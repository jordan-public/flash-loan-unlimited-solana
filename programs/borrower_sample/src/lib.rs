// SPDX-License-Identifier: BUSL-1.1
use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Mint, Token, TokenAccount, Transfer}};

declare_id!("BiBiaMTWecRB3cbz6oMfrKq3F1VKCRLCxKgS3NYLTMCK");

#[program]
pub mod borrower_sample {
    use super::*;

    pub fn create_accounts(_ctx: Context<CreateAccounts>) -> Result<()> {
        Ok(())
    }

    pub fn handle_borrow(ctx: Context<HandleBorrow>, amount: u64) -> Result<()> {
        // Assume the borrowed amount is available in the borrower's PDA account
        msg!("Borrowed amount: {}", amount);

        // Put your business logic here
        // - Use the borrowed amount to perform some business logic
        // - Send the profit to the user_account

        // Calculate the fee (0.3% of the borrowed amount)
        let fee = (amount as u64 * 25) / 1000; // Simplified calculation for 0.25%
        let total_repayment = amount + fee;
        require!(total_repayment <= ctx.accounts.borrower_account.amount, ErrorCode::InsufficientFunds);

        // Repay the borrowed amount and the fee to the lender
        let cpi_accounts = Transfer {
            from: ctx.accounts.borrower_account.to_account_info(),
            to: ctx.accounts.lender_account.to_account_info(),
            authority: ctx.accounts.borrower_account.to_account_info(),
        };
        let mint_key = ctx.accounts.mint.key();
        let seeds = &[
            b"borrower_account",
            mint_key.as_ref(),
            &[ctx.bumps.borrower_account],
        ];
        let signer = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, total_repayment)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct CreateAccounts<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(init_if_needed, payer = user, token::mint = mint, token::authority = borrower_account, seeds = [b"borrower_account".as_ref(), mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub borrower_account: Account<'info, TokenAccount>,
    /// CHECK: This is only used to validate the token mint, not for storage
    pub mint: Account<'info, Mint>,  
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct HandleBorrow<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(mut, token::mint = mint, token::authority = borrower_account, seeds = [b"borrower_account".as_ref(), mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub borrower_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lender_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}


#[error_code]
pub enum ErrorCode {
    // ... (existing error codes)
    #[msg("Insufficient funds")]
    InsufficientFunds,
}