mod errors;
mod instructions;
mod state;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("4p7QHt9TRS9a2teThJzWP8q6eyRAkL5WojxcHN9eEavg");

#[program]
pub mod doug_stake {
    use super::*;

    pub fn init(ctx: Context<Init>) -> Result<()> {
        ctx.accounts.handler()
    }

    pub fn stake(ctx: Context<Stake>, amount: u64, duration: u64) -> Result<()> {
        ctx.accounts.handler(amount, duration, ctx.bumps.user_vault)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.handler(ctx.bumps.user_vault, ctx.bumps.reward_vault)
    }

    pub fn restake(ctx: Context<Restake>) -> Result<()> {
        ctx.accounts.handler(ctx.bumps.reward_vault)
    }

    pub fn extend(ctx: Context<Extend>, duration: u64) -> Result<()> {
        ctx.accounts.handler(duration)
    }
}
