use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Burn, Token, TokenAccount, Transfer};
use std::mem::size_of;
use solana_program::sysvar::rent::Rent;
use borrower_sample::cpi::accounts::HandleBorrow;
use borrower_sample::program::BorrowerSample;
use borrower_sample::{self};

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

        let pool_balance = ctx.accounts.pool_account.amount;
        let fluf_total_supply = ctx.accounts.fluf_mint.supply;
        let amount_fluf = if pool_balance == 0 { 
            amount 
        } else { 
            let numerator = (amount as u128) * (fluf_total_supply as u128);
            let denominator = pool_balance as u128;
            (numerator / denominator) as u64
        };

        // Transfer the pool token to the pool PDA (initialize this PDA if it doesn't exist)
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_account.to_account_info(),
            to: ctx.accounts.pool_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let program_cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(program_cpi_ctx, amount)?; // Fails if the user doesn't have enough tokens

        // Mint fluf tokens to the user
        let cpi_accounts = MintTo {
            mint: ctx.accounts.fluf_mint.to_account_info(),
            to: ctx.accounts.user_fluf_account.to_account_info(),
            authority: ctx.accounts.pool.to_account_info(),
        };
        let pool_mint_key = ctx.accounts.pool_mint.key();
        let seeds = &[
            b"pool",
            pool_mint_key.as_ref(),
            &[ctx.bumps.pool],
        ];
        let signer = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, amount_fluf)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        require!(pool.fluf_mint == ctx.accounts.fluf_mint.key(), ErrorCode::InvalidPool);

        let pool_balance = ctx.accounts.pool_account.amount;
        let fluf_total_supply = ctx.accounts.fluf_mint.supply;
        let amount_fluf = ctx.accounts.user_fluf_account.amount;
        let amount = if fluf_total_supply == 0 {
            0
        } else {
            let numerator = (amount_fluf as u128) * (pool_balance as u128);
            let denominator = fluf_total_supply as u128;
            (numerator / denominator) as u64
        };

        // Burn the fluf tokens
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Burn {
            mint: ctx.accounts.fluf_mint.to_account_info(),
            from: ctx.accounts.user_fluf_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount_fluf)?; // Fails if the user doesn't have enough fluf tokens
        
        // Transfer the pool token to the user
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_account.to_account_info(),
            to: ctx.accounts.user_account.to_account_info(),
            authority: ctx.accounts.pool.to_account_info(),
        };
        let pool_mint_key = ctx.accounts.pool_mint.key();
        let seeds = &[
            b"pool",
            pool_mint_key.as_ref(),
            &[ctx.bumps.pool],
        ];
        let signer = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
    
    pub fn lend_and_call(ctx: Context<LendAndCall>, amount: u64) -> Result<()> {
        // Make sure the pool PDA exists
        let pool = &ctx.accounts.pool;
        require!(pool.pool_mint == ctx.accounts.pool_mint.key(), ErrorCode::InvalidPool);
        require!(pool.fluf_mint == ctx.accounts.fluf_mint.key(), ErrorCode::InvalidPool);

        // Make sure the pool is not empty - otherwise fees cannot be paid
        require!(ctx.accounts.pool_account.amount > 0, ErrorCode::EmptyPool);

        // Mint fluf tokens to the borrower PDA
        // Mint fluf tokens to the user
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: ctx.accounts.fluf_mint.to_account_info(),
            to: ctx.accounts.borrower_fluf_account.to_account_info(),
            authority: ctx.accounts.pool.to_account_info(),
        };
        let pool_mint_key = ctx.accounts.pool_mint.key();
        let seeds = &[
            b"pool",
            pool_mint_key.as_ref(),
            &[ctx.bumps.pool],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, amount)?;

        // // Call Borrower handle_borrow entry point here
        // let cpi_accounts = HandleBorrow {
        //     user: ctx.accounts.user.to_account_info(),
        //     borrower_account: ctx.accounts.borrower_fluf_account.to_account_info(),
        //     lender_account: ctx.accounts.pool_fluf_account.to_account_info(),
        //     user_account: ctx.accounts.user_fluf_account.to_account_info(),
        //     mint: ctx.accounts.fluf_mint.to_account_info(),
        //     system_program: ctx.accounts.system_program.to_account_info(),
        //     token_program: ctx.accounts.token_program.to_account_info(),
        // };
        // let cpi_program = ctx.accounts.borrower_program.to_account_info();
        // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        //     //.with_remaining_accounts(ctx.remaining_accounts.to_vec());
        // borrower_sample::cpi::handle_borrow(cpi_ctx)?;

        // // Check if loan and fees are paid back
        // // The previous balance of the pool_fluf_account should be 0
        // // ... if this is not the case, someone must have donated to the pool,
        // // ... and this amount should be distributed to all participants as if it were fees
        // require!(ctx.accounts.pool_fluf_account.amount >= amount * 1025 / 1000, ErrorCode::FeesNotPaidBack);

        // // Transfer the proper share to the FLUF Protocol fee account
        // let mut amount_to_burn = ctx.accounts.pool_fluf_account.amount;
        // {
        // let cpi_accounts = Transfer {
        //     from: ctx.accounts.pool_fluf_account.to_account_info(),
        //     to: ctx.accounts.fee_account.to_account_info(),
        //     authority: ctx.accounts.pool.to_account_info(),
        // };
        // let cpi_program = ctx.accounts.token_program.to_account_info();
        // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // // 1000 * 5 / 25 = 200
        // let fee_amount = (ctx.accounts.pool_fluf_account.amount - amount) / 200;
        // amount_to_burn -= fee_amount;
        // token::transfer(cpi_ctx, fee_amount)?;
        // }

        // {
        // // Burn the remaining fluf tokens
        // let cpi_program = ctx.accounts.token_program.to_account_info();
        // let cpi_accounts = Burn {
        //     mint: ctx.accounts.fluf_mint.to_account_info(),
        //     from: ctx.accounts.pool_fluf_account.to_account_info(),
        //     authority: ctx.accounts.pool.to_account_info(),
        // };
        // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // token::burn(cpi_ctx, amount_to_burn)?;
        // }

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
    #[account(mut, token::mint = pool_mint, token::authority = user)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(mut, mint::authority = pool, seeds = [b"fluf_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fluf_mint: Account<'info, Mint>,
    #[account(mut, token::mint = fluf_mint, token::authority = pool, seeds = [b"pool_fluf_account".as_ref(), pool_mint.key().as_ref()], bump,)]
    pub pool_fluf_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = fluf_mint, token::authority = user)]
    pub user_fluf_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(signer, mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(mut, token::mint = pool_mint, token::authority = pool, seeds = [b"pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = pool_mint, token::authority = user)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(mut, mint::authority = pool, seeds = [b"fluf_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fluf_mint: Account<'info, Mint>,
    #[account(mut, token::mint = fluf_mint, token::authority = pool, seeds = [b"pool_fluf_account".as_ref(), pool_mint.key().as_ref()], bump,)]
    pub pool_fluf_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = fluf_mint, token::authority = user)]
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
    #[account(mut, token::mint = pool_mint, token::authority = pool, seeds = [b"pool_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut, mint::authority = pool, seeds = [b"fluf_mint".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fluf_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_fluf_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, token::mint = fluf_mint, token::authority = borrower_fluf_account, seeds = [b"borrower_account".as_ref(), fluf_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub borrower_fluf_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_fluf_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, token::mint = fluf_mint, token::authority = pool, seeds = [b"fee_account".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub fee_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    // Borrower program account
    pub borrower_program: Program<'info, BorrowerSample>,
    // Other accounts (used by the borrower program entry point)
    // &ctx.remaining_accounts does not need declaration - it is automatically included
}

// !!! See above: the account should be owned by the borrower_program
// #[account(init_if_needed, payer = user, token::mint = fluf_mint, token::authority = borrower_fluf_account, seeds = [b"borrower_account".as_ref(), fluf_mint.key().as_ref()], bump, owner = borrower_program.key(), rent_exempt = enforce)]
// pub borrower_fluf_account: Account<'info, TokenAccount>,


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
    #[msg("Fees not paid back")]
    FeesNotPaidBack,
}