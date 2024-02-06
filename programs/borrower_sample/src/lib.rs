use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Mint, Token, TokenAccount, Transfer}, token_interface::spl_token_2022::state::Multisig};

declare_id!("5N7gCufd5hEVkcHVSwtUmAKaHvNNagkq7T4qcUYzJ91y");

#[program]
pub mod borrower_sample {
    use super::*;

    pub fn create_accounts(ctx: Context<CreateAccounts>) -> Result<()> {
        Ok(())
    }

    pub fn handle_borrow(ctx: Context<HandleBorrow>) -> Result<()> {
        // Assume the borrowed amount is available in the borrower's PDA account
        let borrowed_amount = ctx.accounts.borrower_account.amount;
        msg!("Borrowed amount: {}", borrowed_amount);

        // Put your business logic here
        // - Use the borrowed amount to perform some business logic
        // - Send the profit to the user_account

        // Calculate the fee (0.3% of the borrowed amount)
        let fee = (borrowed_amount as u64 * 3) / 1000; // Simplified calculation for 0.3%
        let total_repayment = borrowed_amount + fee;

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
