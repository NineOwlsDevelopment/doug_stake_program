use anchor_lang::prelude::*;
use anchor_spl::{ token::{ Mint, Token, TokenAccount } };
use crate::state::*;

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(
        init,
        seeds = [VAULT_INFO_SEED],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<VaultInfo>()
    )]
    pub vault_info: Box<Account<'info, VaultInfo>>,

    #[account(
        init,
        seeds = [REWARD_VAULT_SEED],
        bump,
        payer = user,
        token::mint = reward_token_mint,
        token::authority = reward_vault
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub reward_token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Init<'info> {
    pub fn handler(&mut self) -> Result<()> {
        self.vault_info.is_initialized = true;
        self.vault_info.total_value_locked = 0;
        self.vault_info.lifetime_value_locked = 0;
        Ok(())
    }
}
