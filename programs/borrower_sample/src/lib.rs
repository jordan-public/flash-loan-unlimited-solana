use anchor_lang::prelude::*;

declare_id!("5N7gCufd5hEVkcHVSwtUmAKaHvNNagkq7T4qcUYzJ91y");

#[program]
pub mod borrower_sample {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
