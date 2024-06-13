use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ transfer, Mint, Token, TokenAccount, Transfer },
};

#[derive(Accounts)]
pub struct TopUp<'info> {
    #[account(
        mut,
        seeds = [VAULT_INFO_SEED],
        bump,
    )]
    pub vault_info: Box<Account<'info, VaultInfo>>,

    #[account(
        mut,
        seeds = [USER_VAULT_SEED, user.key().as_ref()],
        bump,
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [STAKE_ACCOUNT_SEED, user.key().as_ref()],
        bump,
    )]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> TopUp<'info> {
    pub fn handler(&mut self, amount: u64) -> Result<()> {
        self.stake_account.top_up(self.mint.key(), amount)?;

        // Transfer the tokens from the user's wallet to their vault
        let cpi_context = CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.user_token_account.to_account_info(),
            to: self.user_vault.to_account_info(),
            authority: self.user.to_account_info(),
        });

        transfer(cpi_context, amount)?;

        self.vault_info.total_value_locked += amount;
        self.vault_info.lifetime_value_locked += amount;

        Ok(())
    }
}
