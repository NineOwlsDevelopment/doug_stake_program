use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ transfer, Mint, Token, TokenAccount, Transfer },
};

#[derive(Accounts)]
pub struct Unstake<'info> {
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

    #[account(
        mut,
        seeds = [REWARD_VAULT_SEED],
        bump,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Unstake<'info> {
    pub fn handler(&mut self, vault_bump: u8, reward_bump: u8) -> Result<()> {
        // transfer staked amount from user's vault to user's wallet
        let user_key = self.user.key();
        let signer: &[&[&[u8]]] = &[&[USER_VAULT_SEED, user_key.as_ref(), &[vault_bump]]];
        let cpi_context = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_vault.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: self.user_vault.to_account_info(),
            },
            signer
        );
        let staked_amount = self.stake_account.amount;
        transfer(cpi_context, staked_amount)?;

        // transfer rewards from reward vault to user's wallet
        let signer: &[&[&[u8]]] = &[&[REWARD_VAULT_SEED, &[reward_bump]]];
        let cpi_context = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reward_vault.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: self.reward_vault.to_account_info(),
            },
            signer
        );
        let reward_amount = self.stake_account.rewards;
        transfer(cpi_context, reward_amount)?;

        self.stake_account.unstake()?;
        self.vault_info.total_value_locked -= staked_amount;
        Ok(())
    }
}
