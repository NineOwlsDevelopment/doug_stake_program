use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{ transfer, Token, TokenAccount, Transfer };

#[derive(Accounts)]
pub struct Restake<'info> {
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
        seeds = [REWARD_VAULT_SEED],
        bump,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Restake<'info> {
    pub fn handler(&mut self, reward_bump: u8) -> Result<()> {
        // send the tokens from the reward vault to the users vault
        let signer: &[&[&[u8]]] = &[&[REWARD_VAULT_SEED, &[reward_bump]]];
        let cpi_context = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reward_vault.to_account_info(),
                to: self.user_vault.to_account_info(),
                authority: self.reward_vault.to_account_info(),
            },
            signer
        );

        let reward_amount = self.stake_account.rewards;
        transfer(cpi_context, reward_amount)?;

        self.vault_info.total_value_locked += self.stake_account.amount;
        self.stake_account.restake()?;
        Ok(())
    }
}
