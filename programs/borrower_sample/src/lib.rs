use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("5N7gCufd5hEVkcHVSwtUmAKaHvNNagkq7T4qcUYzJ91y");

#[program]
pub mod borrower_sample {
    use super::*;

    pub fn handle_borrow(ctx: Context<HandleBorrow>, _mint_address: Pubkey) -> Result<()> {
        // Assume the borrowed amount is available in the borrower's PDA account
        let borrowed_amount = ctx.accounts.borrower_pda_account.amount;
        msg!("Borrowed amount: {}", borrowed_amount);

        // Put your business logic here

        // Calculate the fee (0.3% of the borrowed amount)
        let fee = (borrowed_amount as u64 * 3) / 1000; // Simplified calculation for 0.3%
        let total_repayment = borrowed_amount + fee;

        // Perform the transfer (CPI call) directly in this function
        let cpi_accounts = Transfer {
            from: ctx.accounts.borrower_pda_account.to_account_info(),
            to: ctx.accounts.lender_pda_account.to_account_info(),
            authority: ctx.accounts.borrower_pda_account.to_account_info(),
            
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_repayment)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct HandleBorrow<'info> {
    #[account(mut)]
    pub borrower_pda_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lender_pda_account: Account<'info, TokenAccount>,
    /// CHECK: This is only used to validate the token mint, not for storage
    pub mint_address: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}
