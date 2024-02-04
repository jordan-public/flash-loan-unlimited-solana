use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, InitializeMint, mint_to};
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
        pool.wrapped_mint = ctx.accounts.wrapped_mint.key();
        pool.voucher_mint = ctx.accounts.voucher_mint.key();
        
        // Make sure the number of decimals matches the pool mint (and thus the wrapped and voucher mints as well)
        let pool_mint_decimals = ctx.accounts.pool_mint.decimals;
        require!(pool_mint_decimals == decimals, ErrorCode::DecimalsMismatch);

        // Print the pool token mint address
        msg!("Pool token mint: {}", pool.pool_mint);
        msg!("Wrapped mint: {:?}", pool.wrapped_mint);
        msg!("Voucher mint: {}", pool.voucher_mint);

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        // Not needed: require!(pool.wrapped_mint == ctx.accounts.wrapped_mint.key(), ErrorCode::InvalidPool);
        require!(pool.voucher_mint == ctx.accounts.voucher_mint.key(), ErrorCode::InvalidPool);

        // Transfer the pool token to the pool PDA (initialize this PDA if it doesn't exist)

        // Record/update user's deposit value factor (pool token amount / voucher token amount)

        // Mint voucher tokens to the user

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        // Not needed: require!(pool.wrapped_mint == ctx.accounts.wrapped_mint.key(), ErrorCode::InvalidPool);
        require!(pool.voucher_mint == ctx.accounts.voucher_mint.key(), ErrorCode::InvalidPool);

        // Calculate the user's pool_mint amount based on the voucher token amount and the pool value factor
        // Pool value factor = pool token amount / voucher token amount

        // Burn the user's voucher tokens

        // Transfer pool_mint tokens to the user

        // Update the user's deposit value factor

        Ok(())
    }

    pub fn wrap(ctx: Context<Wrap>, amount: u64) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        require!(pool.wrapped_mint == ctx.accounts.wrapped_mint.key(), ErrorCode::InvalidPool);
        // Not needed: require!(pool.voucher_mint == ctx.accounts.voucher_mint.key(), ErrorCode::InvalidPool);

        // Transfer pool_mint tokens to the pool PDA

        // Mint wrapped tokens to the user in the same amount

        Ok(())
    }

    pub fn unwrap(ctx: Context<Unwrap>, amount: u64) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        require!(pool.wrapped_mint == ctx.accounts.wrapped_mint.key(), ErrorCode::InvalidPool);
        // Not needed: require!(pool.voucher_mint == ctx.accounts.voucher_mint.key(), ErrorCode::InvalidPool);

        // Burn wrapped tokens from the user

        // Transfer same amount of pool_mint tokens to the user

        Ok(())
    }

    pub fn lend_and_call(ctx: Context<LendAndCall>, amount: u64, wrapped: bool) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        require!(pool.wrapped_mint == ctx.accounts.wrapped_mint.key(), ErrorCode::InvalidPool);
        require!(pool.voucher_mint == ctx.accounts.voucher_mint.key(), ErrorCode::InvalidPool);

        // Make sure the pool is not empty - otherwise fees cannot be paid
        // Determine the total mint of voucher tokens
        let total_voucher_mint = 0; // TODO:
        // Check for empty pool
        require!(total_voucher_mint > 0, ErrorCode::EmptyPool);

        // Transfer pool_mint or wrapped tokens to the borrower PDA

        // Call Borrower borrow() entry point

        // Check if loan and fees are paid back

        // Unwrap and burn any reccieved wrapped tokens

        Ok(())
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        // Make sure the owner is the caller
        require!(ctx.accounts.user.key() == ctx.accounts.state.deployer, ErrorCode::InvalidAdmin);
        
        // Transfer all fees to the collector account

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
    pub wrapped_mint: Pubkey, // Address of the wrapped token's mint (if any),
    pub voucher_mint: Pubkey, // Address of the voucher token's mint (if any)
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
    #[account(init, payer = user, mint::authority = pool, mint::decimals = decimals, seeds = [b"wrapped_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub wrapped_mint: Account<'info, Mint>,
    #[account(init, payer = user, token::mint = wrapped_mint, token::authority = pool, seeds = [b"wrapped_pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub wrapped_pool_account: Account<'info, TokenAccount>,
    #[account(init, payer = user, mint::authority = pool, mint::decimals = decimals, seeds = [b"voucher_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub voucher_mint: Account<'info, Mint>,
    // There is no need to create a voucher pool account, as it only minted but never held by the pool
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
    #[account(seeds = [b"voucher_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub voucher_mint: Account<'info, Mint>,
    #[account(mut, token::mint = voucher_mint, token::authority = pool, seeds = [b"pool_voucher_account".as_ref(), pool_mint.key().as_ref()], bump,)]
    pub pool_voucher_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, token::mint = voucher_mint, token::authority = user, seeds = [b"user_voucher_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump,)]
    pub user_voucher_account: Account<'info, TokenAccount>,
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
    #[account(seeds = [b"voucher_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub voucher_mint: Account<'info, Mint>,
    #[account(mut, token::mint = voucher_mint, token::authority = pool, seeds = [b"pool_voucher_account".as_ref(), pool_mint.key().as_ref()], bump,)]
    pub pool_voucher_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = voucher_mint, token::authority = user, seeds = [b"user_voucher_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump,)]
    pub user_voucher_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Wrap<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(mut, token::mint = pool_mint, token::authority = pool, seeds = [b"pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = user, seeds = [b"user_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump, rent_exempt = enforce)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"wrapped_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub wrapped_mint: Account<'info, Mint>,
    #[account(mut, token::mint = wrapped_mint, token::authority = pool, seeds = [b"pool_wrapped_account".as_ref(), pool_mint.key().as_ref()], bump,)]
    pub pool_wrapped_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, token::mint = wrapped_mint, token::authority = user, seeds = [b"user_wrapped_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump,)]
    pub user_wrapped_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Unwrap<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(mut, token::mint = pool_mint, token::authority = pool, seeds = [b"pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, token::mint = pool_mint, token::authority = user, seeds = [b"user_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump, rent_exempt = enforce)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"wrapped_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub wrapped_mint: Account<'info, Mint>,
    #[account(mut, token::mint = wrapped_mint, token::authority = pool, seeds = [b"pool_wrapped_account".as_ref(), pool_mint.key().as_ref()], bump,)]
    pub pool_wrapped_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = wrapped_mint, token::authority = user, seeds = [b"user_wrapped_account".as_ref(), pool_mint.key().as_ref(), user.key().as_ref()], bump,)]
    pub user_wrapped_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64, wrapped: bool)]
pub struct LendAndCall<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(mut, token::mint = pool_mint, token::authority = pool, seeds = [b"pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub borrower_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, token::mint = pool_mint, token::authority = pool, seeds = [b"fer_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fee_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"wrapped".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub wrapped_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_wrapped_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub borrower_wrapped_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"voucher_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub voucher_mint: Account<'info, Mint>,
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
    #[account(mut, token::mint = pool_mint, token::authority = pool, seeds = [b"fee_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
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