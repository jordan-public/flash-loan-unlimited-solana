use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, InitializeMint, mint_to};
use std::mem::size_of;
use solana_program::sysvar::rent::Rent;

declare_id!("7Crsw9yaDiT5jMZ8yWJgkdVeWpLirh9G5hJZCp9G1Aiy");

#[program]
mod fluf {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_pool(ctx: Context<CreatePool>, wrapped_mode: bool) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.pool_mint = ctx.accounts.pool_mint.key();
        pool.wrapped_mode = wrapped_mode;

        // Set the mint authority to the program's PDA
        let mint_authority = ctx.program_id.key();

        // Discover the decimals and name of the pool token
        let pool_mint_decimals = ctx.accounts.pool_mint.decimals;

        if wrapped_mode {
            // Create wrapped token mint
            let wrapped_mint = &mut ctx.accounts.wrapped_mint;
            // Initialize the mint for the wrapped token
            let cpi_accounts = InitializeMint {
                mint: wrapped_mint.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::initialize_mint(cpi_ctx, pool_mint_decimals, &mint_authority, None)?;

            pool.wrapped_mint = Some(wrapped_mint.key());
        }

        // Create voucher token mint
        let voucher_mint = &mut ctx.accounts.voucher_mint;
        // Initialize the mint for the voucher token
        let cpi_accounts = InitializeMint {
            mint: voucher_mint.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::initialize_mint(cpi_ctx, pool_mint_decimals, &mint_authority, None)?;

        pool.voucher_mint = voucher_mint.key();

        Ok(())
    }
}

#[account]
pub struct Pool {
    pub pool_mint: Pubkey, // Address of the pool token's mint
    pub wrapped_mint: Option<Pubkey>, // Address of the wrapped token's mint (if any),
    pub voucher_mint: Pubkey, // Address of the voucher token's mint (if any)
    pub wrapped_mode: bool, // Indicates if the pool is in wrapped mode
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
#[instruction(wrapped_mode: bool)]
pub struct CreatePool<'info> {
    #[account(signer, mut)]
    pub initializer: Signer<'info>,
    #[account(init, payer = initializer, space = 8 + size_of::<Pool>(), seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub pool_mint: Account<'info, Mint>,
    #[account(init, payer = initializer, space = Mint::LEN, seeds = [b"wrapped".as_ref(), pool_mint.key().as_ref()], bump)]
    pub wrapped_mint: Account<'info, Mint>,
    #[account(init, payer = initializer, space = Mint::LEN, seeds = [b"voucher".as_ref(), pool_mint.key().as_ref()], bump)]
    pub voucher_mint: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
