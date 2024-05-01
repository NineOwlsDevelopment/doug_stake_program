use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Extend<'info> {
    #[account(
        mut,
        seeds = [STAKE_ACCOUNT_SEED, user.key().as_ref()],
        bump,
    )]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Extend<'info> {
    pub fn handler(&mut self, duration: u64) -> Result<()> {
        self.stake_account.extend(duration)?;
        Ok(())
    }
}
