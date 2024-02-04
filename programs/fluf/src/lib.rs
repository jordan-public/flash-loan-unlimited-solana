use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, InitializeMint, mint_to};
use std::mem::size_of;
use solana_program::sysvar::rent::Rent;
use solana_program::pubkey;

declare_id!("7Crsw9yaDiT5jMZ8yWJgkdVeWpLirh9G5hJZCp9G1Aiy");
const HARDCODED_PUBKEY: Pubkey = pubkey!("7Crsw9yaDiT5jMZ8yWJgkdVeWpLirh9G5hJZCp9G1Aiy");

#[program]
mod fluf {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_pool(ctx: Context<CreatePool>, decimals: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.pool_mint = ctx.accounts.pool_mint.key();
        pool.wrapped_mint = ctx.accounts.wrapped_mint.key();
        pool.voucher_mint = ctx.accounts.voucher_mint.key();
        
        // Set the mint authority to the program's PDA
        let mint_authority = ctx.program_id.key();
        require!(mint_authority == HARDCODED_PUBKEY, ErrorCode::InvalidMintAuthority);
        require!(ctx.accounts.wrapped_mint.mint_authority == solana_program::program_option::COption::Some(mint_authority), ErrorCode::MintAuthorityMismatch);
        require!(ctx.accounts.voucher_mint.mint_authority == solana_program::program_option::COption::Some(mint_authority), ErrorCode::MintAuthorityMismatch);

        // Discover the decimals and name of the pool token
        let pool_mint_decimals = ctx.accounts.pool_mint.decimals;
        require!(pool_mint_decimals == decimals, ErrorCode::DecimalsMismatch);

        // Print the pool token mint address
        msg!("Pool token mint: {}", pool.pool_mint);
        msg!("Wrapped mint: {:?}", pool.wrapped_mint);
        msg!("Voucher mint: {}", pool.voucher_mint);

        Ok(())
    }
}

#[account]
pub struct Pool {
    pub pool_mint: Pubkey, // Address of the pool token's mint
    pub wrapped_mint: Pubkey, // Address of the wrapped token's mint (if any),
    pub voucher_mint: Pubkey, // Address of the voucher token's mint (if any)
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct CreatePool<'info> {
    #[account(signer, mut)]
    pub initializer: Signer<'info>,
    #[account(init, payer = initializer, space = 8 + size_of::<Pool>(), seeds = [b"pool".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub pool: Account<'info, Pool>,
    pub pool_mint: Account<'info, Mint>,
    #[account(init, payer = initializer, mint::authority = HARDCODED_PUBKEY, mint::decimals = decimals, seeds = [b"wrapped".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub wrapped_mint: Account<'info, Mint>,
    #[account(init, payer = initializer, mint::authority = HARDCODED_PUBKEY, mint::decimals = decimals, seeds = [b"voucher".as_ref(), pool_mint.key().as_ref()], bump, rent_exempt = enforce)]
    pub voucher_mint: Account<'info, Mint>,
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
}