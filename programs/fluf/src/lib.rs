use anchor_lang::prelude::*;

declare_id!("2qxVaPhpX8XxNGyNZrnNe4UKD8q2dVNC5NL1WzCnGxaL");

#[program]
pub mod fluf {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
