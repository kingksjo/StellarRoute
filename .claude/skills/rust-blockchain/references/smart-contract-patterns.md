# Smart Contract Design Patterns

## Table of Contents
- [Structural Patterns](#structural-patterns)
- [Behavioral Patterns](#behavioral-patterns)
- [Security Patterns](#security-patterns)
- [Economic Patterns](#economic-patterns)
- [Upgradeability Patterns](#upgradeability-patterns)
- [Gas/Fee Optimization](#gasfee-optimization)

## Structural Patterns

### Factory Pattern
Create multiple instances of contracts:

```rust
// Substrate Factory Pattern
#[pallet::storage]
pub type Contracts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::Hash,
    ContractMetadata<T::AccountId>,
>;

#[pallet::storage]
pub type ContractCount<T> = StorageValue<_, u64, ValueQuery>;

pub fn create_contract(
    origin: OriginFor<T>,
    init_data: Vec<u8>,
) -> DispatchResult {
    let creator = ensure_signed(origin)?;
    
    let contract_id = Self::generate_contract_id(&creator);
    let contract = ContractMetadata {
        creator: creator.clone(),
        created_at: <frame_system::Pallet<T>>::block_number(),
        data: init_data,
    };
    
    Contracts::<T>::insert(contract_id, contract);
    ContractCount::<T>::mutate(|count| *count += 1);
    
    Self::deposit_event(Event::ContractCreated { 
        id: contract_id, 
        creator 
    });
    
    Ok(())
}

// Solana Factory Pattern
#[program]
pub mod factory {
    use super::*;

    pub fn create_instance(
        ctx: Context<CreateInstance>,
        seed: u64,
        bump: u8,
    ) -> Result<()> {
        let instance = &mut ctx.accounts.instance;
        instance.creator = ctx.accounts.creator.key();
        instance.seed = seed;
        instance.bump = bump;
        instance.created_at = Clock::get()?.unix_timestamp;
        
        emit!(InstanceCreated {
            instance: instance.key(),
            creator: instance.creator,
        });
        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(seed: u64, bump: u8)]
pub struct CreateInstance<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Instance::INIT_SPACE,
        seeds = [b"instance", creator.key().as_ref(), &seed.to_le_bytes()],
        bump,
    )]
    pub instance: Account<'info, Instance>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
```

### Registry Pattern
Maintain a registry of approved contracts/addresses:

```rust
// Registry of verified contracts
#[pallet::storage]
pub type VerifiedContracts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::Hash,
    ContractInfo,
    OptionQuery,
>;

#[pallet::storage]
pub type Registrar<T: Config> = StorageValue<_, T::AccountId>;

pub fn register_contract(
    origin: OriginFor<T>,
    contract_id: T::Hash,
    info: ContractInfo,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let registrar = Registrar::<T>::get().ok_or(Error::<T>::NoRegistrar)?;
    
    ensure!(who == registrar, Error::<T>::NotRegistrar);
    
    VerifiedContracts::<T>::insert(contract_id, info);
    Self::deposit_event(Event::ContractRegistered { contract_id });
    
    Ok(())
}

pub fn is_verified(contract_id: &T::Hash) -> bool {
    VerifiedContracts::<T>::contains_key(contract_id)
}

// Solana Registry
#[account]
#[derive(InitSpace)]
pub struct Registry {
    pub authority: Pubkey,
    pub verified_programs: Vec<Pubkey>,
}

impl Registry {
    pub fn is_verified(&self, program: &Pubkey) -> bool {
        self.verified_programs.contains(program)
    }
}
```

### Proxy Pattern
Delegate calls to implementation contract:

```rust
// Substrate Proxy Pattern
#[pallet::storage]
pub type Implementation<T: Config> = StorageValue<_, T::Hash>;

#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn upgrade_implementation(
        origin: OriginFor<T>,
        new_impl: T::Hash,
    ) -> DispatchResult {
        ensure_root(origin)?;
        Implementation::<T>::put(new_impl);
        Self::deposit_event(Event::ImplementationUpgraded { new_impl });
        Ok(())
    }
    
    pub fn delegate_call(
        origin: OriginFor<T>,
        call_data: Vec<u8>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        let impl_hash = Implementation::<T>::get()
            .ok_or(Error::<T>::NoImplementation)?;
        
        // Delegate to implementation
        // (Specific implementation depends on your framework)
        
        Ok(())
    }
}
```

## Behavioral Patterns

### State Machine Pattern
Model contract lifecycle with explicit states:

```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ContractState {
    Initialized,
    Active,
    Paused,
    Finalized,
}

#[account]
#[derive(InitSpace)]
pub struct StatefulContract {
    pub state: ContractState,
    pub admin: Pubkey,
    pub data: u64,
}

impl StatefulContract {
    pub fn ensure_state(&self, expected: ContractState) -> Result<()> {
        require!(self.state == expected, ErrorCode::InvalidState);
        Ok(())
    }
    
    pub fn transition_to(&mut self, new_state: ContractState) -> Result<()> {
        // Validate state transitions
        let valid_transition = match (&self.state, &new_state) {
            (ContractState::Initialized, ContractState::Active) => true,
            (ContractState::Active, ContractState::Paused) => true,
            (ContractState::Paused, ContractState::Active) => true,
            (ContractState::Active, ContractState::Finalized) => true,
            _ => false,
        };
        
        require!(valid_transition, ErrorCode::InvalidStateTransition);
        self.state = new_state;
        Ok(())
    }
}

pub fn activate(ctx: Context<Activate>) -> Result<()> {
    let contract = &mut ctx.accounts.contract;
    contract.ensure_state(ContractState::Initialized)?;
    contract.transition_to(ContractState::Active)?;
    Ok(())
}

pub fn pause(ctx: Context<Pause>) -> Result<()> {
    let contract = &mut ctx.accounts.contract;
    contract.ensure_state(ContractState::Active)?;
    contract.transition_to(ContractState::Paused)?;
    Ok(())
}
```

### Oracle Pattern
Fetch off-chain data securely:

```rust
// Substrate Off-chain Worker
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn offchain_worker(block_number: T::BlockNumber) {
        if let Err(e) = Self::fetch_price_and_send() {
            log::error!("Error fetching price: {:?}", e);
        }
    }
}

impl<T: Config> Pallet<T> {
    fn fetch_price_and_send() -> Result<(), &'static str> {
        // Fetch data from external API
        let price = Self::fetch_external_price()?;
        
        // Create signed transaction to submit on-chain
        let call = Call::submit_price { price };
        
        // Submit unsigned transaction with signed payload
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|_| "Failed to submit transaction")?;
        
        Ok(())
    }
    
    fn fetch_external_price() -> Result<u64, &'static str> {
        // HTTP request to external API
        let request = http::Request::get("https://api.example.com/price")
            .send()
            .map_err(|_| "HTTP request failed")?;
        
        let response = request.wait()
            .map_err(|_| "Failed to get response")?;
        
        let price = parse_price(response.body())
            .map_err(|_| "Failed to parse price")?;
        
        Ok(price)
    }
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(10_000)]
    pub fn submit_price(
        origin: OriginFor<T>,
        price: u64,
    ) -> DispatchResult {
        ensure_none(origin)?;  // Allow unsigned transactions
        
        // Verify price is reasonable
        let last_price = LastPrice::<T>::get();
        ensure!(
            price_within_bounds(last_price, price),
            Error::<T>::PriceOutOfBounds
        );
        
        LastPrice::<T>::put(price);
        Self::deposit_event(Event::PriceUpdated { price });
        
        Ok(())
    }
}

// Solana Chainlink Oracle Integration
pub fn fetch_price(ctx: Context<FetchPrice>) -> Result<()> {
    let round = chainlink::latest_round_data(
        ctx.accounts.chainlink_feed.to_account_info(),
    )?;
    
    let price = &mut ctx.accounts.price_account;
    price.value = round.answer;
    price.timestamp = round.updated_at;
    
    Ok(())
}
```

### Event Sourcing Pattern
Store state changes as events:

```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum DomainEvent {
    AccountCreated { id: u64, owner: AccountId },
    BalanceUpdated { id: u64, old: u128, new: u128 },
    AccountClosed { id: u64 },
}

#[pallet::storage]
pub type EventLog<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::BlockNumber,
    BoundedVec<DomainEvent, T::MaxEventsPerBlock>,
    ValueQuery,
>;

pub fn record_event(event: DomainEvent) -> DispatchResult {
    let current_block = <frame_system::Pallet<T>>::block_number();
    
    EventLog::<T>::try_mutate(current_block, |events| {
        events.try_push(event)
            .map_err(|_| Error::<T>::TooManyEvents)?;
        Ok(())
    })
}

pub fn replay_events(from_block: T::BlockNumber) -> Vec<DomainEvent> {
    let current_block = <frame_system::Pallet<T>>::block_number();
    
    (from_block..=current_block)
        .filter_map(|block| EventLog::<T>::get(block).into_iter())
        .flatten()
        .collect()
}
```

## Security Patterns

### Pull Payment Pattern
Withdraw funds instead of pushing:

```rust
// ❌ BAD: Push payments (vulnerable to reentrancy)
pub fn distribute_rewards_bad(ctx: Context<Distribute>) -> Result<()> {
    for recipient in &ctx.accounts.recipients {
        // External call in loop - dangerous!
        transfer_tokens(recipient, amount)?;
    }
    Ok(())
}

// ✅ GOOD: Pull payment pattern
#[pallet::storage]
pub type PendingWithdrawals<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BalanceOf<T>,
    ValueQuery,
>;

pub fn allocate_rewards(
    origin: OriginFor<T>,
    recipients: Vec<(T::AccountId, BalanceOf<T>)>,
) -> DispatchResult {
    ensure_root(origin)?;
    
    for (recipient, amount) in recipients {
        PendingWithdrawals::<T>::mutate(&recipient, |balance| {
            *balance = balance.saturating_add(amount);
        });
    }
    
    Ok(())
}

pub fn withdraw(origin: OriginFor<T>) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    let amount = PendingWithdrawals::<T>::take(&who);
    ensure!(amount > Zero::zero(), Error::<T>::NoWithdrawal);
    
    T::Currency::transfer(
        &Self::account_id(),
        &who,
        amount,
        ExistenceRequirement::KeepAlive,
    )?;
    
    Ok(())
}

// Solana Pull Payment
#[account]
#[derive(InitSpace)]
pub struct WithdrawalAccount {
    pub owner: Pubkey,
    pub pending_amount: u64,
}

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let withdrawal = &mut ctx.accounts.withdrawal_account;
    let amount = withdrawal.pending_amount;
    
    require!(amount > 0, ErrorCode::NoPendingWithdrawal);
    
    // Update state FIRST (checks-effects-interactions)
    withdrawal.pending_amount = 0;
    
    // Then transfer
    **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;
    
    Ok(())
}
```

### Circuit Breaker Pattern
Emergency pause functionality:

```rust
#[pallet::storage]
pub type Paused<T> = StorageValue<_, bool, ValueQuery>;

#[pallet::storage]
pub type EmergencyAdmin<T: Config> = StorageValue<_, T::AccountId>;

pub fn pause(origin: OriginFor<T>) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let admin = EmergencyAdmin::<T>::get().ok_or(Error::<T>::NoAdmin)?;
    
    ensure!(who == admin, Error::<T>::NotAdmin);
    
    Paused::<T>::put(true);
    Self::deposit_event(Event::Paused);
    
    Ok(())
}

pub fn unpause(origin: OriginFor<T>) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let admin = EmergencyAdmin::<T>::get().ok_or(Error::<T>::NoAdmin)?;
    
    ensure!(who == admin, Error::<T>::NotAdmin);
    
    Paused::<T>::put(false);
    Self::deposit_event(Event::Unpaused);
    
    Ok(())
}

// Macro for checking pause state
macro_rules! ensure_not_paused {
    () => {
        ensure!(!Paused::<T>::get(), Error::<T>::ContractPaused);
    };
}

pub fn critical_operation(origin: OriginFor<T>) -> DispatchResult {
    ensure_not_paused!();
    let who = ensure_signed(origin)?;
    // Continue with operation
    Ok(())
}

// Solana Circuit Breaker
#[account]
#[derive(InitSpace)]
pub struct CircuitBreaker {
    pub admin: Pubkey,
    pub paused: bool,
}

#[derive(Accounts)]
pub struct Pausable<'info> {
    #[account(
        constraint = !circuit_breaker.paused @ ErrorCode::ContractPaused
    )]
    pub circuit_breaker: Account<'info, CircuitBreaker>,
}
```

### Rate Limiting Pattern
Prevent spam and DOS attacks:

```rust
#[pallet::storage]
pub type RateLimits<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    (u32, T::BlockNumber),  // (count, last_reset_block)
>;

pub fn check_rate_limit(who: &T::AccountId) -> DispatchResult {
    let current_block = <frame_system::Pallet<T>>::block_number();
    let limit_window = T::RateLimitWindow::get();
    let max_calls = T::MaxCallsPerWindow::get();
    
    RateLimits::<T>::try_mutate(who, |limit_opt| {
        let (count, last_reset) = limit_opt.unwrap_or((0, current_block));
        
        // Reset if window expired
        if current_block >= last_reset + limit_window {
            *limit_opt = Some((1, current_block));
            return Ok(());
        }
        
        // Check if limit exceeded
        ensure!(count < max_calls, Error::<T>::RateLimitExceeded);
        
        // Increment count
        *limit_opt = Some((count + 1, last_reset));
        Ok(())
    })
}

pub fn rate_limited_operation(origin: OriginFor<T>) -> DispatchResult {
    let who = ensure_signed(origin)?;
    Self::check_rate_limit(&who)?;
    
    // Continue with operation
    Ok(())
}
```

## Economic Patterns

### Token Vesting Pattern
Gradual token release over time:

```rust
#[account]
#[derive(InitSpace)]
pub struct VestingSchedule {
    pub beneficiary: Pubkey,
    pub total_amount: u64,
    pub start_time: i64,
    pub cliff_duration: i64,
    pub vesting_duration: i64,
    pub amount_withdrawn: u64,
}

impl VestingSchedule {
    pub fn vested_amount(&self, current_time: i64) -> u64 {
        if current_time < self.start_time + self.cliff_duration {
            return 0;
        }
        
        if current_time >= self.start_time + self.vesting_duration {
            return self.total_amount;
        }
        
        let time_vested = current_time - self.start_time;
        (self.total_amount as u128 * time_vested as u128 / self.vesting_duration as u128) as u64
    }
    
    pub fn withdrawable_amount(&self, current_time: i64) -> u64 {
        self.vested_amount(current_time)
            .saturating_sub(self.amount_withdrawn)
    }
}

pub fn withdraw_vested(ctx: Context<WithdrawVested>) -> Result<()> {
    let schedule = &mut ctx.accounts.vesting_schedule;
    let clock = Clock::get()?;
    
    let amount = schedule.withdrawable_amount(clock.unix_timestamp);
    require!(amount > 0, ErrorCode::NothingToWithdraw);
    
    schedule.amount_withdrawn = schedule.amount_withdrawn
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;
    
    // Transfer tokens
    transfer_tokens(ctx, amount)?;
    
    Ok(())
}
```

### Staking Pattern
Lock tokens to earn rewards:

```rust
#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount_staked: u64,
    pub reward_debt: u64,
    pub last_stake_time: i64,
}

#[account]
#[derive(InitSpace)]
pub struct StakingPool {
    pub total_staked: u64,
    pub reward_per_token: u64,
    pub last_update_time: i64,
    pub reward_rate: u64,
}

impl StakingPool {
    pub fn update_rewards(&mut self, current_time: i64) {
        if self.total_staked == 0 {
            self.last_update_time = current_time;
            return;
        }
        
        let time_elapsed = current_time - self.last_update_time;
        let rewards = (self.reward_rate as u128 * time_elapsed as u128) / self.total_staked as u128;
        
        self.reward_per_token = self.reward_per_token
            .checked_add(rewards as u64)
            .unwrap();
        self.last_update_time = current_time;
    }
    
    pub fn calculate_pending_reward(&self, stake: &StakeAccount) -> u64 {
        (stake.amount_staked as u128 * 
         (self.reward_per_token - stake.reward_debt) as u128 / 1e18 as u128) as u64
    }
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let stake_account = &mut ctx.accounts.stake_account;
    let clock = Clock::get()?;
    
    pool.update_rewards(clock.unix_timestamp);
    
    // Claim pending rewards if any
    let pending = pool.calculate_pending_reward(stake_account);
    if pending > 0 {
        transfer_rewards(ctx, pending)?;
    }
    
    // Update stake
    stake_account.amount_staked = stake_account.amount_staked
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;
    stake_account.reward_debt = pool.reward_per_token;
    stake_account.last_stake_time = clock.unix_timestamp;
    
    pool.total_staked = pool.total_staked
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;
    
    Ok(())
}
```

## Upgradeability Patterns

### Data Separation Pattern
Separate logic from data:

```rust
// Data contract (never upgraded)
#[account]
#[derive(InitSpace)]
pub struct DataStore {
    pub logic_contract: Pubkey,
    pub owner: Pubkey,
    pub data: Vec<u64>,
}

// Logic contract (can be upgraded)
pub fn execute_logic(ctx: Context<ExecuteLogic>) -> Result<()> {
    let data_store = &mut ctx.accounts.data_store;
    
    // Verify caller is authorized logic contract
    require!(
        ctx.program_id == &data_store.logic_contract,
        ErrorCode::UnauthorizedLogic
    );
    
    // Execute business logic
    process_data(&mut data_store.data)?;
    
    Ok(())
}

pub fn upgrade_logic(ctx: Context<UpgradeLogic>, new_logic: Pubkey) -> Result<()> {
    let data_store = &mut ctx.accounts.data_store;
    
    require!(
        ctx.accounts.owner.key() == data_store.owner,
        ErrorCode::Unauthorized
    );
    
    data_store.logic_contract = new_logic;
    
    Ok(())
}
```

## Gas/Fee Optimization

### Batch Operations
Process multiple items efficiently:

```rust
// ✅ Efficient: Batch processing
pub fn batch_transfer(
    ctx: Context<BatchTransfer>,
    recipients: Vec<(Pubkey, u64)>,
) -> Result<()> {
    require!(recipients.len() <= MAX_BATCH_SIZE, ErrorCode::BatchTooLarge);
    
    let total: u64 = recipients.iter().map(|(_, amt)| amt).sum();
    require!(
        ctx.accounts.sender_balance.amount >= total,
        ErrorCode::InsufficientBalance
    );
    
    for (recipient, amount) in recipients {
        transfer_internal(&ctx, &recipient, amount)?;
    }
    
    Ok(())
}

// Substrate batch calls
pub fn batch_calls(
    origin: OriginFor<T>,
    calls: Vec<Box<<T as Config>::Call>>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    for call in calls {
        let _ = call.dispatch(RawOrigin::Signed(who.clone()).into());
    }
    
    Ok(())
}
```

### Storage Optimization
Minimize storage usage:

```rust
// ✅ Pack data efficiently
#[account]
pub struct OptimizedAccount {
    pub flags: u8,           // Pack 8 boolean flags
    pub small_value: u16,    // Use smallest type needed
    pub timestamp: i64,      // Only store what's needed
}

impl OptimizedAccount {
    pub fn get_flag(&self, index: u8) -> bool {
        (self.flags & (1 << index)) != 0
    }
    
    pub fn set_flag(&mut self, index: u8, value: bool) {
        if value {
            self.flags |= 1 << index;
        } else {
            self.flags &= !(1 << index);
        }
    }
}
```

These patterns form the foundation for building robust, secure, and efficient blockchain applications.
