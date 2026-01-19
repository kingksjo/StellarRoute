# Blockchain Security Best Practices

## Table of Contents
- [Security Principles](#security-principles)
- [Common Vulnerabilities](#common-vulnerabilities)
- [Input Validation](#input-validation)
- [Access Control](#access-control)
- [Arithmetic Safety](#arithmetic-safety)
- [Storage Security](#storage-security)
- [Cross-Program Security](#cross-program-security)
- [Cryptographic Best Practices](#cryptographic-best-practices)
- [Audit Checklist](#audit-checklist)

## Security Principles

### Defense in Depth
Layer multiple security controls to protect against failures:

```rust
// ✅ Multiple validation layers
#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(10_000)]
    pub fn transfer(
        origin: OriginFor<T>,
        to: T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // Layer 1: Origin validation
        let from = ensure_signed(origin)?;
        
        // Layer 2: Business logic validation
        ensure!(from != to, Error::<T>::SelfTransfer);
        ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);
        
        // Layer 3: Balance check
        let balance = Balances::<T>::get(&from);
        ensure!(balance >= amount, Error::<T>::InsufficientBalance);
        
        // Layer 4: Overflow protection
        let new_to_balance = Balances::<T>::get(&to)
            .checked_add(&amount)
            .ok_or(Error::<T>::Overflow)?;
        
        // Execute transfer
        Balances::<T>::insert(&from, balance.checked_sub(&amount).unwrap());
        Balances::<T>::insert(&to, new_to_balance);
        
        Ok(())
    }
}
```

### Principle of Least Privilege
Grant minimal necessary permissions:

```rust
// ✅ Separate roles with specific permissions
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Role {
    Admin,
    Operator,
    User,
}

pub fn check_permission(role: &Role, action: &Action) -> bool {
    match (role, action) {
        (Role::Admin, _) => true,
        (Role::Operator, Action::Pause | Action::Unpause) => true,
        (Role::User, Action::Transfer | Action::View) => true,
        _ => false,
    }
}
```

### Fail Securely
Errors should fail closed, not open:

```rust
// ✅ Fail securely
pub fn validate_signature(msg: &[u8], sig: &Signature, pubkey: &Pubkey) -> Result<()> {
    // If verification fails, return error (fail closed)
    verify_signature(msg, sig, pubkey)
        .map_err(|_| Error::InvalidSignature)?;
    Ok(())
}

// ❌ Fail insecurely
pub fn validate_signature_bad(msg: &[u8], sig: &Signature, pubkey: &Pubkey) -> bool {
    // If verification fails, might return true (fail open)
    verify_signature(msg, sig, pubkey).is_ok()
}
```

## Common Vulnerabilities

### Reentrancy Attacks

**Vulnerable Code:**
```rust
// ❌ VULNERABLE: External call before state update
pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let account = &ctx.accounts.user_account;
    
    // ❌ External transfer BEFORE updating state
    transfer_tokens(&ctx, amount)?;
    
    // State update happens after external call
    account.balance -= amount;
    Ok(())
}
```

**Secure Code:**
```rust
// ✅ SECURE: Update state before external calls
pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.user_account;
    
    // ✅ Update state FIRST
    require!(account.balance >= amount, ErrorCode::InsufficientBalance);
    account.balance = account.balance
        .checked_sub(amount)
        .ok_or(ErrorCode::Overflow)?;
    
    // Then make external call
    transfer_tokens(&ctx, amount)?;
    Ok(())
}
```

### Integer Overflow/Underflow

**Vulnerable Code:**
```rust
// ❌ VULNERABLE: Unchecked arithmetic
pub fn add_balance(ctx: Context<AddBalance>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.user_account;
    account.balance += amount;  // ❌ Can overflow!
    Ok(())
}
```

**Secure Code:**
```rust
// ✅ SECURE: Checked arithmetic
pub fn add_balance(ctx: Context<AddBalance>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.user_account;
    account.balance = account.balance
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;
    Ok(())
}

// ✅ Alternative: Use SafeMath trait
use sp_arithmetic::traits::{CheckedAdd, CheckedSub, CheckedMul, CheckedDiv};

pub fn safe_arithmetic(a: u128, b: u128) -> Result<u128, Error> {
    let sum = a.checked_add(b).ok_or(Error::Overflow)?;
    let diff = sum.checked_sub(10).ok_or(Error::Underflow)?;
    let product = diff.checked_mul(2).ok_or(Error::Overflow)?;
    Ok(product)
}
```

### Unauthorized Access

**Vulnerable Code:**
```rust
// ❌ VULNERABLE: No authority check
pub fn update_config(ctx: Context<UpdateConfig>, value: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.value = value;  // ❌ Anyone can call this!
    Ok(())
}
```

**Secure Code:**
```rust
// ✅ SECURE: Verify authority
#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        has_one = authority,  // Verify config.authority matches signer
    )]
    pub config: Account<'info, Config>,
    
    pub authority: Signer<'info>,
}

pub fn update_config(ctx: Context<UpdateConfig>, value: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.value = value;  // ✅ Only authority can call
    Ok(())
}

// Substrate equivalent
#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn update_config(origin: OriginFor<T>, value: u64) -> DispatchResult {
        // ✅ Ensure origin is root/admin
        ensure_root(origin)?;
        // or
        let who = ensure_signed(origin)?;
        ensure!(Self::is_admin(&who), Error::<T>::NotAuthorized);
        
        ConfigValue::<T>::put(value);
        Ok(())
    }
}
```

### Account Substitution

**Vulnerable Code:**
```rust
// ❌ VULNERABLE: No account validation
#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from: Account<'info, UserAccount>,
    #[account(mut)]
    pub to: Account<'info, UserAccount>,
    pub authority: Signer<'info>,  // ❌ Not validated against 'from'
}
```

**Secure Code:**
```rust
// ✅ SECURE: Validate account ownership
#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(
        mut,
        has_one = authority,  // ✅ Verify from.authority == authority
    )]
    pub from: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub to: Account<'info, UserAccount>,
    
    pub authority: Signer<'info>,
}

// Or use seeds/bump for PDA validation
#[derive(Accounts)]
pub struct TransferPda<'info> {
    #[account(
        mut,
        seeds = [b"user", authority.key().as_ref()],
        bump = from.bump,
    )]
    pub from: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub to: Account<'info, UserAccount>,
    
    pub authority: Signer<'info>,
}
```

## Input Validation

### Comprehensive Validation
```rust
// ✅ Validate all inputs
pub fn create_item(
    ctx: Context<CreateItem>,
    name: String,
    price: u64,
    quantity: u32,
) -> Result<()> {
    // Validate name
    require!(!name.is_empty(), ErrorCode::EmptyName);
    require!(name.len() <= MAX_NAME_LENGTH, ErrorCode::NameTooLong);
    require!(is_valid_utf8(&name), ErrorCode::InvalidEncoding);
    
    // Validate price
    require!(price > 0, ErrorCode::InvalidPrice);
    require!(price <= MAX_PRICE, ErrorCode::PriceTooHigh);
    
    // Validate quantity
    require!(quantity > 0, ErrorCode::InvalidQuantity);
    require!(quantity <= MAX_QUANTITY, ErrorCode::QuantityTooHigh);
    
    // Validate total value won't overflow
    let total = (price as u128)
        .checked_mul(quantity as u128)
        .ok_or(ErrorCode::Overflow)?;
    require!(total <= u64::MAX as u128, ErrorCode::TotalTooLarge);
    
    let item = &mut ctx.accounts.item;
    item.name = name;
    item.price = price;
    item.quantity = quantity;
    
    Ok(())
}

// Substrate equivalent
#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn create_item(
        origin: OriginFor<T>,
        name: Vec<u8>,
        price: BalanceOf<T>,
        quantity: u32,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        // Validate name
        ensure!(!name.is_empty(), Error::<T>::EmptyName);
        ensure!(name.len() <= MAX_NAME_LENGTH, Error::<T>::NameTooLong);
        let bounded_name: BoundedVec<u8, T::MaxNameLength> = 
            name.try_into().map_err(|_| Error::<T>::NameTooLong)?;
        
        // Validate price
        ensure!(price > Zero::zero(), Error::<T>::InvalidPrice);
        
        // Validate quantity
        ensure!(quantity > 0, Error::<T>::InvalidQuantity);
        ensure!(quantity <= T::MaxQuantity::get(), Error::<T>::QuantityTooHigh);
        
        Items::<T>::insert(&who, Item {
            name: bounded_name,
            price,
            quantity,
        });
        
        Ok(())
    }
}
```

### Sanitize External Data
```rust
// ✅ Sanitize strings from external sources
pub fn sanitize_string(input: &str) -> Result<String> {
    // Remove control characters
    let sanitized: String = input
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect();
    
    // Validate UTF-8
    ensure!(sanitized.is_ascii() || is_valid_utf8(&sanitized), Error::InvalidEncoding);
    
    // Trim whitespace
    let trimmed = sanitized.trim();
    
    // Check length
    ensure!(!trimmed.is_empty(), Error::EmptyString);
    ensure!(trimmed.len() <= MAX_LENGTH, Error::TooLong);
    
    Ok(trimmed.to_string())
}
```

## Access Control

### Role-Based Access Control (RBAC)
```rust
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AccessControl {
    pub super_admin: Pubkey,
    pub admins: Vec<Pubkey>,
    pub operators: Vec<Pubkey>,
}

impl AccessControl {
    pub fn is_super_admin(&self, pubkey: &Pubkey) -> bool {
        self.super_admin == *pubkey
    }
    
    pub fn is_admin(&self, pubkey: &Pubkey) -> bool {
        self.is_super_admin(pubkey) || self.admins.contains(pubkey)
    }
    
    pub fn is_operator(&self, pubkey: &Pubkey) -> bool {
        self.is_admin(pubkey) || self.operators.contains(pubkey)
    }
}

// Usage in instructions
pub fn admin_only(ctx: Context<AdminOnly>) -> Result<()> {
    let access_control = &ctx.accounts.access_control;
    require!(
        access_control.is_admin(&ctx.accounts.authority.key()),
        ErrorCode::Unauthorized
    );
    
    // Admin-only logic
    Ok(())
}

// Substrate RBAC
use frame_support::traits::EnsureOrigin;

pub enum Role {
    Root,
    Admin,
    Operator,
}

#[pallet::storage]
pub type Admins<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), ValueQuery>;

#[pallet::storage]
pub type Operators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), ValueQuery>;

impl<T: Config> Pallet<T> {
    pub fn is_admin(who: &T::AccountId) -> bool {
        Admins::<T>::contains_key(who)
    }
    
    pub fn ensure_admin(who: &T::AccountId) -> DispatchResult {
        ensure!(Self::is_admin(who), Error::<T>::NotAdmin);
        Ok(())
    }
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    // Root only
    pub fn root_operation(origin: OriginFor<T>) -> DispatchResult {
        ensure_root(origin)?;
        // Root logic
        Ok(())
    }
    
    // Admin only
    pub fn admin_operation(origin: OriginFor<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;
        Self::ensure_admin(&who)?;
        // Admin logic
        Ok(())
    }
}
```

### Time-Based Access Control
```rust
// ✅ Time-locked operations
#[account]
#[derive(InitSpace)]
pub struct TimeLock {
    pub owner: Pubkey,
    pub locked_until: i64,
    pub amount: u64,
}

pub fn withdraw_timelock(ctx: Context<WithdrawTimeLock>) -> Result<()> {
    let timelock = &ctx.accounts.timelock;
    let clock = Clock::get()?;
    
    require!(
        clock.unix_timestamp >= timelock.locked_until,
        ErrorCode::StillLocked
    );
    
    // Proceed with withdrawal
    Ok(())
}

// Substrate timelock
#[pallet::storage]
pub type TimeLocks<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    (BalanceOf<T>, T::BlockNumber),
>;

pub fn withdraw_timelock(origin: OriginFor<T>) -> DispatchResult {
    let who = ensure_signed(origin)?;
    let current_block = <frame_system::Pallet<T>>::block_number();
    
    let (amount, unlock_block) = TimeLocks::<T>::get(&who)
        .ok_or(Error::<T>::NoTimelock)?;
    
    ensure!(current_block >= unlock_block, Error::<T>::StillLocked);
    
    // Proceed with withdrawal
    Ok(())
}
```

## Arithmetic Safety

### Always Use Checked Operations
```rust
// ✅ Comprehensive safe math
pub fn safe_calculations(a: u64, b: u64, c: u64) -> Result<u64> {
    // Addition
    let sum = a.checked_add(b).ok_or(ErrorCode::Overflow)?;
    
    // Subtraction
    let diff = sum.checked_sub(c).ok_or(ErrorCode::Underflow)?;
    
    // Multiplication
    let product = diff.checked_mul(2).ok_or(ErrorCode::Overflow)?;
    
    // Division
    let quotient = product.checked_div(b).ok_or(ErrorCode::DivisionByZero)?;
    
    // Modulo
    let remainder = quotient.checked_rem(c).ok_or(ErrorCode::DivisionByZero)?;
    
    Ok(remainder)
}

// Fixed-point arithmetic for precision
use sp_arithmetic::{FixedI128, FixedU128};

pub fn calculate_interest(principal: u128, rate_percent: u128) -> Result<u128> {
    // Use fixed-point for decimal calculations
    let principal = FixedU128::from_inner(principal);
    let rate = FixedU128::from_rational(rate_percent, 100);
    
    let interest = principal.checked_mul(&rate)
        .ok_or(Error::Overflow)?;
    
    Ok(interest.into_inner())
}
```

## Storage Security

### Prevent Storage Exhaustion
```rust
// ✅ Bounded collections in Substrate
use frame_support::BoundedVec;

#[pallet::storage]
pub type UserData<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u8, T::MaxDataSize>,
>;

// ✅ Limit iterations
pub fn process_users(max_count: u32) -> Result<()> {
    let count = Users::<T>::iter()
        .take(max_count as usize)
        .count();
    Ok(())
}

// ✅ Require deposits for storage
pub fn store_data(ctx: Context<StoreData>, data: Vec<u8>) -> Result<()> {
    // Calculate rent
    let rent = Rent::get()?;
    let space_needed = 8 + 32 + 4 + data.len();
    let rent_amount = rent.minimum_balance(space_needed);
    
    // Verify user has enough balance
    require!(
        ctx.accounts.user.lamports() >= rent_amount,
        ErrorCode::InsufficientRent
    );
    
    // Store data
    Ok(())
}
```

## Cross-Program Security

### Validate Program IDs
```rust
// ✅ Verify called program
pub fn safe_cpi(ctx: Context<SafeCpi>) -> Result<()> {
    // Verify the program being called
    require!(
        ctx.accounts.called_program.key() == EXPECTED_PROGRAM_ID,
        ErrorCode::InvalidProgram
    );
    
    // Proceed with CPI
    Ok(())
}

// ✅ Validate account ownership
pub fn verify_account_owner(account: &AccountInfo, expected_owner: &Pubkey) -> Result<()> {
    require!(account.owner == expected_owner, ErrorCode::InvalidOwner);
    Ok(())
}
```

## Cryptographic Best Practices

### Secure Random Number Generation
```rust
// ✅ Secure randomness in Substrate
use frame_support::traits::Randomness;

pub fn generate_random_id(
    who: &T::AccountId,
    nonce: u64,
) -> T::Hash {
    let (random_seed, _) = T::Randomness::random(&nonce.encode());
    let input = (random_seed, who, nonce).encode();
    T::Hashing::hash(&input)
}

// Solana: Use Verifiable Random Function (VRF)
// Or request randomness from Chainlink VRF
```

### Signature Verification
```rust
// ✅ Verify signatures correctly
use solana_program::ed25519_program;

pub fn verify_ed25519_signature(
    message: &[u8],
    signature: &[u8; 64],
    public_key: &[u8; 32],
) -> Result<()> {
    // Construct instruction data for ed25519 verification
    let ix = ed25519_program::new_ed25519_instruction(
        public_key,
        message,
        signature,
    );
    
    // Verify signature is valid
    require!(
        ed25519_program::verify(&ix),
        ErrorCode::InvalidSignature
    );
    
    Ok(())
}
```

## Audit Checklist

### Pre-Deployment Checklist
```
Security Audit Checklist:

□ Input Validation
  □ All inputs validated for type, range, and format
  □ String lengths bounded
  □ No unbounded loops or recursion
  □ Array/vector access bounds-checked

□ Access Control
  □ All sensitive functions require authentication
  □ Authority checks on all state-changing operations
  □ Role-based permissions implemented correctly
  □ No privilege escalation vulnerabilities

□ Arithmetic Safety
  □ All arithmetic uses checked operations
  □ No integer overflow/underflow possible
  □ Division by zero prevented
  □ Fixed-point arithmetic for decimals

□ Storage Security
  □ Bounded collections used everywhere
  □ Storage deposits required
  □ No unbounded storage growth
  □ Efficient storage layout

□ Reentrancy Protection
  □ State updated before external calls
  □ No reentrancy vulnerabilities
  □ Mutex/locks where needed

□ Account Security
  □ Account ownership verified
  □ Signer requirements enforced
  □ PDA seeds/bumps validated
  □ Account substitution prevented

□ Cross-Program Invocation
  □ Program IDs verified
  □ Account ownership checked in CPI
  □ Return values validated
  □ Privilege escalation prevented

□ Testing
  □ Unit tests cover all functions
  □ Integration tests for workflows
  □ Fuzz testing for inputs
  □ Security tests for vulnerabilities

□ Documentation
  □ Security considerations documented
  □ Known limitations listed
  □ Upgrade process defined
  □ Emergency procedures documented
```

### Automated Security Tools
```bash
# Run Cargo audit
cargo audit

# Run Clippy with strict lints
cargo clippy -- -D warnings

# Substrate-specific checks
cargo dylint --all -- --all-features

# Solana-specific tools
anchor test
solana-test-validator

# Static analysis
cargo geiger  # Unsafe code detection

# Fuzzing
cargo fuzz run fuzz_target_1
```

This provides a comprehensive security foundation for blockchain development in Rust.
