use anchor_lang::prelude::*;
use crate::errors;

pub const USER_VAULT_SEED: &[u8] = b"user_vault";
pub const STAKE_ACCOUNT_SEED: &[u8] = b"stake_account";
pub const VAULT_INFO_SEED: &[u8] = b"vault_info";
pub const REWARD_VAULT_SEED: &[u8] = b"reward_vault";

pub const DECIMALS_PER_TOKEN: u64 = 1000000;
pub const STAKE_MINIMUM: u64 = 100 * DECIMALS_PER_TOKEN;
pub const SECONDS_PER_DAY: u64 = 60; // 60 for test case only
pub const DURATION_MIN: u64 = 14;
pub const DURATION_MAX: u64 = 365;

#[account]
pub struct VaultInfo {
    pub total_value_locked: u64,
    pub lifetime_value_locked: u64,
    pub is_initialized: bool,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount: u64,
    pub rewards: u64,
    pub duration: u64,
    pub vault: Pubkey,
    pub vault_bump: u8,
    pub unlockable_at: i64,
    pub is_staked: bool,
}

impl StakeAccount {
    pub fn stake(
        &mut self,
        owner: Pubkey,
        amount: u64,
        duration: u64,
        vault: Pubkey,
        vault_bump: u8
    ) -> Result<()> {
        if self.is_staked {
            return Err(errors::ErrorCode::AlreadyStaked.into());
        }

        if amount < STAKE_MINIMUM {
            return Err(errors::ErrorCode::AmountNotEnough.into());
        }

        if duration < DURATION_MIN || duration > DURATION_MAX {
            return Err(errors::ErrorCode::DurationTooShort.into());
        }

        self.owner = owner;
        self.amount = amount;
        self.rewards = self.calculate_rewards(duration);
        self.vault = vault;
        self.vault_bump = vault_bump;
        self.duration = duration;
        self.unlockable_at = self.calculate_unlockable_at(duration).unwrap();
        self.is_staked = true;
        Ok(())
    }

    pub fn unstake(&mut self) -> Result<()> {
        if !self.is_staked {
            return Err(errors::ErrorCode::NotStaked.into());
        }

        if self.unlockable_at > Clock::get()?.unix_timestamp {
            return Err(errors::ErrorCode::Locked.into());
        }

        self.amount = 0;
        self.unlockable_at = 0;
        self.rewards = 0;
        self.is_staked = false;
        Ok(())
    }

    pub fn restake(&mut self) -> Result<()> {
        if !self.is_staked {
            return Err(errors::ErrorCode::NotStaked.into());
        }

        if self.unlockable_at > Clock::get()?.unix_timestamp {
            return Err(errors::ErrorCode::Locked.into());
        }

        self.amount += self.rewards;
        self.rewards = self.calculate_rewards(self.duration);
        self.unlockable_at = self.calculate_unlockable_at(self.duration).unwrap();
        Ok(())
    }

    pub fn extend(&mut self, duration: u64) -> Result<()> {
        if !self.is_staked {
            return Err(errors::ErrorCode::NotStaked.into());
        }

        if duration <= 0 {
            return Err(errors::ErrorCode::DurationTooShort.into());
        }

        if self.unlockable_at < Clock::get()?.unix_timestamp {
            return Err(errors::ErrorCode::AlreadyUnlockable.into());
        }

        if self.duration + duration <= 0 {
            return Err(errors::ErrorCode::DurationTooShort.into());
        }

        if self.duration + duration > DURATION_MAX {
            return Err(errors::ErrorCode::DurationTooLong.into());
        }

        self.duration += duration;
        self.unlockable_at = self.calculate_unlockable_at(self.duration).unwrap();
        self.rewards = self.calculate_rewards(self.duration);
        Ok(())
    }

    fn calculate_rewards(&self, duration: u64) -> u64 {
        let seconds_per_year = 86400 * 365;
        let unstake_time = 86400 * duration;
        let multiplier = ((unstake_time as f64) / (seconds_per_year as f64)) * 3.0 + 1.0;
        let rewards = ((self.amount as f64) * multiplier).ceil();
        (rewards as u64) - self.amount
    }

    fn calculate_unlockable_at(&self, duration: u64) -> Result<i64> {
        let now = Clock::get()?.unix_timestamp;
        Ok(now + ((SECONDS_PER_DAY * duration) as i64))
    }
}
