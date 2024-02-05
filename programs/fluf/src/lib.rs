use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, InitializeMint, mint_to};
use std::mem::size_of;
use solana_program::sysvar::rent::Rent;
use solana_program::pubkey;

declare_id!("7Crsw9yaDiT5jMZ8yWJgkdVeWpLirh9G5hJZCp9G1Aiy");

#[program]
mod fluf {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Initialize the program state - it can be done only once, since the state account is marked as account(init, ...)
        let state = &mut ctx.accounts.state;
        state.deployer = ctx.accounts.deployer.key();
        
        Ok(())
    }

    pub fn create_pool(ctx: Context<CreatePool>, decimals: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.pool_mint = ctx.accounts.pool_mint.key();
        pool.fluf_mint = ctx.accounts.fluf_mint.key();
        
        // Make sure the number of decimals matches the pool mint (and thus fluf mints as well)
        let pool_mint_decimals = ctx.accounts.pool_mint.decimals;
        require!(pool_mint_decimals == decimals, ErrorCode::DecimalsMismatch);

        // Print the pool token mint address
        msg!("Pool token mint: {}", pool.pool_mint);
        msg!("FLUF mint: {}", pool.fluf_mint);

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        require!(pool.fluf_mint == ctx.accounts.fluf_mint.key(), ErrorCode::InvalidPool);

        // Transfer the pool token to the pool PDA (initialize this PDA if it doesn't exist)

        // Record/update user's deposit value factor (pool token amount / fluf token amount)

        // Mint fluf tokens to the user

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        require!(pool.fluf_mint == ctx.accounts.fluf_mint.key(), ErrorCode::InvalidPool);

        // Calculate the user's pool_mint amount based on the fluf token amount and the pool value factor
        // Pool value factor = pool token amount / fluf token amount

        // Burn the user's fluf tokens

        // Transfer pool_mint tokens to the user

        // Update the user's deposit value factor

        Ok(())
    }

    pub fn lend_and_call(ctx: Context<LendAndCall>, amount: u64) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        require!(pool.fluf_mint == ctx.accounts.fluf_mint.key(), ErrorCode::InvalidPool);

        // Make sure the pool is not empty - otherwise fees cannot be paid
        // Determine the total mint of fluf tokens
        let total_fluf_mint = 0; // TODO:
        // Check for empty pool
        require!(total_fluf_mint > 0, ErrorCode::EmptyPool);

        // Transfer pool_mint or fluf tokens to the borrower PDA

        // Call Borrower borrow() entry point

        // Check if loan and fees are paid back

        // Keep the fluf tokens (re-invested in the pool)

        Ok(())
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        // Make sure the owner is the caller
        require!(ctx.accounts.user.key() == ctx.accounts.state.deployer, ErrorCode::InvalidAdmin);
        
        // Make sure the pool PDA matches the provided fluf mint
        let pool = &ctx.accounts.pool;
        require!(pool.fluf_mint == ctx.accounts.fluf_mint.key(), ErrorCode::InvalidPool);

        // Obtain the fee_account balance
        let balance = ctx.accounts.fee_account.amount;
        
        // Prepare the transfer instruction
        let cpi_accounts = Transfer {
            from: ctx.accounts.fee_account.to_account_info(),
            to: ctx.accounts.collector_account.to_account_info(),
            authority: ctx.accounts.pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        // Execute the transfer of the entire balance of the fee account for the pool mint
        token::transfer(cpi_ctx, balance)?;

        msg!("Fees from {} fluf pool withdrawn: {}", ctx.accounts.fluf_mint.key(), balance);

        Ok(())
    }

}

#[account]
pub struct ProgramState {
    deployer: Pubkey,
}

#[account]
pub struct Pool {
    pub pool_mint: Pubkey, // Address of the pool token's mint
    pub fluf_mint: Pubkey, // Address of the fluf token's mint (if any)
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // Record the deployer of the program (for administative purposes)
    #[account(signer, mut)]
    pub deployer: Signer<'info>,
    #[account(init, payer = deployer, space = 8 + size_of::<ProgramState>(), seeds = [b"program_state".as_ref()], bump, rent_exempt = enforce)]
    pub state: Account<'info, ProgramState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct CreatePool<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(init, payer = user, space = 8 + size_of::<Pool>(), seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>, // This is the mint of the pool token - it can be any SPL token
    #[account(init, payer = user, token::mint = pool_mint, token::authority = pool, seeds = [b"pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(init, payer = user, mint::authority = pool, mint::decimals = decimals, seeds = [b"fluf_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fluf_mint: Account<'info, Mint>,
    #[account(init, payer = user, token::mint = fluf_mint, token::authority = pool, seeds = [b"pool_fluf_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_fluf_account: Account<'info, TokenAccount>,
    // There is no need to create a fluf pool account, as it only minted but never held by the pool
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Deposit<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(mut, token::mint = pool_mint, token::authority = pool, seeds = [b"pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = user, seeds = [b"user_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump, rent_exempt = enforce)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"fluf_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fluf_mint: Account<'info, Mint>,
    #[account(mut, token::mint = fluf_mint, token::authority = pool, seeds = [b"pool_fluf_account".as_ref(), pool_mint.key().as_ref()], bump,)]
    pub pool_fluf_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, token::mint = fluf_mint, token::authority = user, seeds = [b"user_fluf_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump,)]
    pub user_fluf_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(mut, token::mint = pool_mint, token::authority = pool, seeds = [b"pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = user, seeds = [b"user_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump, rent_exempt = enforce)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"fluf_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fluf_mint: Account<'info, Mint>,
    #[account(mut, token::mint = fluf_mint, token::authority = pool, seeds = [b"pool_fluf_account".as_ref(), pool_mint.key().as_ref()], bump,)]
    pub pool_fluf_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = fluf_mint, token::authority = user, seeds = [b"user_fluf_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump,)]
    pub user_fluf_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct LendAndCall<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(seeds = [b"fluf_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fluf_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_fluf_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub borrower_fluf_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, token::mint = fluf_mint, token::authority = pool, seeds = [b"fee_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fee_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    // Borrower program account
    /// CHECK: This is the borrower program account that will be called - it is safe because at the end we check for the repayment
    pub borrower_program: AccountInfo<'info>,
    // Other accounts (used by the borrower program entry point)
    // &ctx.remaining_accounts does not need declaration - it is automatically included
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct WithdrawFees<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"program_state".as_ref()], bump, rent_exempt = enforce)]
    pub state: Account<'info, ProgramState>,
    #[account(seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(seeds = [b"fluf_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fluf_mint: Account<'info, Mint>,
    #[account(mut, token::mint = fluf_mint, token::authority = pool, seeds = [b"fee_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fee_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub collector_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    // ... (existing error codes)
    #[msg("Pool mint decimals must match the provided decimals")]
    DecimalsMismatch,
    #[msg("Mint authority must be the program's PDA")]
    InvalidMintAuthority,
    #[msg("Mint authority mismatch")]
    MintAuthorityMismatch,
    #[msg("Invalid pool")]
    InvalidPool,
    #[msg("Empty pool")]
    EmptyPool,
    #[msg("Unauthorized")]
    InvalidAdmin,
}