use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct StakingContract {
    owner: AccountId,
    stake_balances: UnorderedMap<AccountId, u128>,
    total_staked: u128,
}

#[near_bindgen]
impl StakingContract {
    #[init]
    pub fn new() -> Self {
        Self {
            owner: env::predecessor_account_id(),
            stake_balances: UnorderedMap::new(b"s".to_vec()),
            total_staked: 0,
        }
    }

    #[payable]
    pub fn stake(&mut self) -> u128 {
        let deposit = env::attached_deposit();
        let user = env::predecessor_account_id();
        log!("{} is staking {}", user, deposit);

        match self.stake_balances.get(&user) {
            Some(balance) => {
                let new_balance = balance.saturating_add(u128::from(deposit));
                self.stake_balances.insert(&user, &new_balance);
                self.total_staked = self.total_staked.saturating_add(deposit);
                new_balance
            }
            None => {
                let new_balance = u128::from(deposit);
                self.stake_balances.insert(&user, &new_balance);
                self.total_staked = self.total_staked.saturating_add(deposit);
                new_balance
            }
        }
    }

    pub fn unstake(&mut self, amount: U128) -> bool {
        assert!(u128::from(amount) > 0);
        let user = env::predecessor_account_id();
        log!("{} is unstaking {}", user, u128::from(amount));

        match self.stake_balances.get(&user) {
            Some(balance) => {
                let new_balance = balance.saturating_sub(u128::from(amount));
                self.stake_balances.insert(&user, &new_balance);
                self.total_staked = self.total_staked.saturating_sub(u128::from(amount));
                if new_balance == 0 {
                    //User unstaked all their balance, so refund it all
                    Promise::new(user).transfer(balance);
                } else {
                    //User unstaked a portion of their balance, refund just that
                    Promise::new(user).transfer(u128::from(amount));
                }
                true
            }
            _ => false,
        }
    }

    pub fn airdrop(&mut self, amount: u128) {
        let user = env::predecessor_account_id();
        assert!(user == self.owner);
        for (staker, _) in self.stake_balances.iter() {
            Promise::new(staker).transfer(amount);
        }
    }

    pub fn get_total_staked(&self) -> u128 {
        self.total_staked
    }

    pub fn get_user_staked(&self) -> u128 {
        let user = env::predecessor_account_id();
        match self.stake_balances.get(&user) {
            Some(balance) => balance,
            None => 0,
        }
    }

    pub fn get_total_balance(&self) -> Balance {
        env::account_balance()
    }

    pub fn get_account_id(&self) -> AccountId {
        env::current_account_id()
    }
}
