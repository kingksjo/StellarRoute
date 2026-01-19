# Testing and Deployment Strategies

## Table of Contents
- [Testing Philosophy](#testing-philosophy)
- [Unit Testing](#unit-testing)
- [Integration Testing](#integration-testing)
- [Property-Based Testing](#property-based-testing)
- [Fuzzing](#fuzzing)
- [Load Testing](#load-testing)
- [Security Testing](#security-testing)
- [Deployment Strategies](#deployment-strategies)
- [Monitoring and Maintenance](#monitoring-and-maintenance)

## Testing Philosophy

### Test Pyramid for Blockchain
```
        ╱╲
       ╱  ╲      E2E Tests (Few)
      ╱────╲     - Full workflow tests
     ╱      ╲    - Cross-contract integration
    ╱────────╲   
   ╱          ╲  Integration Tests (Some)
  ╱────────────╲ - Pallet/program integration
 ╱              ╲- Cross-module tests
╱────────────────╲
                  Unit Tests (Many)
                  - Function-level tests
                  - Edge cases
                  - Error conditions
```

### Testing Priorities
1. **Security-critical paths**: Authentication, authorization, value transfers
2. **State transitions**: Ensure state changes are correct and atomic
3. **Edge cases**: Boundary values, zero amounts, maximum values
4. **Error handling**: All error paths covered
5. **Invariants**: Properties that must always hold true

## Unit Testing

### Substrate Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use frame_support::{assert_ok, assert_noop, assert_err};
    use sp_runtime::traits::BadOrigin;

    #[test]
    fn create_profile_works() {
        new_test_ext().execute_with(|| {
            // Arrange
            let account = 1;
            let name = vec![b'A'; 10];
            let initial_balance = Balances::free_balance(account);

            // Act
            assert_ok!(MyPallet::create_profile(
                RuntimeOrigin::signed(account),
                name.clone()
            ));

            // Assert
            assert_eq!(Profiles::<Test>::get(account).unwrap().to_vec(), name);
            assert_eq!(
                Balances::free_balance(account),
                initial_balance - MinDeposit::get()
            );
            
            // Verify event
            System::assert_has_event(
                Event::ProfileCreated { who: account }.into()
            );
        });
    }

    #[test]
    fn create_profile_fails_if_exists() {
        new_test_ext().execute_with(|| {
            let account = 1;
            let name = vec![b'A'; 10];
            
            assert_ok!(MyPallet::create_profile(
                RuntimeOrigin::signed(account),
                name.clone()
            ));
            
            assert_noop!(
                MyPallet::create_profile(
                    RuntimeOrigin::signed(account),
                    name
                ),
                Error::<Test>::ProfileAlreadyExists
            );
        });
    }

    #[test]
    fn create_profile_fails_if_name_too_long() {
        new_test_ext().execute_with(|| {
            let account = 1;
            let name = vec![b'A'; (MaxNameLength::get() + 1) as usize];
            
            assert_err!(
                MyPallet::create_profile(
                    RuntimeOrigin::signed(account),
                    name
                ),
                Error::<Test>::NameTooLong
            );
        });
    }

    #[test]
    fn create_profile_fails_if_insufficient_balance() {
        new_test_ext().execute_with(|| {
            let poor_account = 999;  // Not in genesis config
            let name = vec![b'A'; 10];
            
            assert_noop!(
                MyPallet::create_profile(
                    RuntimeOrigin::signed(poor_account),
                    name
                ),
                pallet_balances::Error::<Test>::InsufficientBalance
            );
        });
    }

    #[test]
    fn remove_profile_returns_deposit() {
        new_test_ext().execute_with(|| {
            let account = 1;
            let name = vec![b'A'; 10];
            
            assert_ok!(MyPallet::create_profile(
                RuntimeOrigin::signed(account),
                name
            ));
            
            let balance_after_create = Balances::free_balance(account);
            
            assert_ok!(MyPallet::remove_profile(
                RuntimeOrigin::signed(account)
            ));
            
            assert_eq!(
                Balances::free_balance(account),
                balance_after_create + MinDeposit::get()
            );
        });
    }

    #[test]
    fn test_invariants() {
        new_test_ext().execute_with(|| {
            // Test: Total supply never changes
            let initial_supply = Balances::total_issuance();
            
            // Perform various operations
            assert_ok!(MyPallet::create_profile(
                RuntimeOrigin::signed(1),
                vec![b'A'; 10]
            ));
            
            assert_eq!(Balances::total_issuance(), initial_supply);
        });
    }
}

// Mock runtime for testing
// See substrate-development.md for complete mock implementation
```

### Solana/Anchor Unit Tests
```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MyProgram } from "../target/types/my_program";
import { expect } from "chai";
import { Keypair, PublicKey } from "@solana/web3.js";

describe("my-program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.MyProgram as Program<MyProgram>;
  
  let userAccount: Keypair;
  let profilePda: PublicKey;
  let bump: number;

  beforeEach(async () => {
    userAccount = Keypair.generate();
    
    // Airdrop SOL for testing
    const signature = await provider.connection.requestAirdrop(
      userAccount.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);
    
    // Find PDA
    [profilePda, bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from("profile"),
        userAccount.publicKey.toBuffer(),
      ],
      program.programId
    );
  });

  describe("createProfile", () => {
    it("creates a profile successfully", async () => {
      const name = "John Doe";
      
      const tx = await program.methods
        .createProfile(name, bump)
        .accounts({
          profile: profilePda,
          user: userAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([userAccount])
        .rpc();

      const profile = await program.account.profile.fetch(profilePda);
      expect(profile.name).to.equal(name);
      expect(profile.owner.toString()).to.equal(userAccount.publicKey.toString());
    });

    it("fails if name is too long", async () => {
      const longName = "A".repeat(100);
      
      try {
        await program.methods
          .createProfile(longName, bump)
          .accounts({
            profile: profilePda,
            user: userAccount.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([userAccount])
          .rpc();
        
        expect.fail("Should have thrown error");
      } catch (err) {
        expect(err.error.errorCode.code).to.equal("NameTooLong");
      }
    });

    it("fails if profile already exists", async () => {
      const name = "John Doe";
      
      // Create profile first time
      await program.methods
        .createProfile(name, bump)
        .accounts({
          profile: profilePda,
          user: userAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([userAccount])
        .rpc();
      
      // Try to create again
      try {
        await program.methods
          .createProfile(name, bump)
          .accounts({
            profile: profilePda,
            user: userAccount.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([userAccount])
          .rpc();
        
        expect.fail("Should have thrown error");
      } catch (err) {
        // Account already exists error
        expect(err).to.exist;
      }
    });
  });

  describe("updateProfile", () => {
    beforeEach(async () => {
      await program.methods
        .createProfile("Original Name", bump)
        .accounts({
          profile: profilePda,
          user: userAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([userAccount])
        .rpc();
    });

    it("updates profile successfully", async () => {
      const newName = "Updated Name";
      
      await program.methods
        .updateProfile(newName)
        .accounts({
          profile: profilePda,
          user: userAccount.publicKey,
        })
        .signers([userAccount])
        .rpc();

      const profile = await program.account.profile.fetch(profilePda);
      expect(profile.name).to.equal(newName);
    });

    it("fails if not owner", async () => {
      const attacker = Keypair.generate();
      
      // Airdrop to attacker
      const sig = await provider.connection.requestAirdrop(
        attacker.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);
      
      try {
        await program.methods
          .updateProfile("Hacked Name")
          .accounts({
            profile: profilePda,
            user: attacker.publicKey,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail("Should have thrown error");
      } catch (err) {
        expect(err.error.errorCode.code).to.equal("Unauthorized");
      }
    });
  });

  describe("closeProfile", () => {
    it("closes profile and returns rent", async () => {
      await program.methods
        .createProfile("Test", bump)
        .accounts({
          profile: profilePda,
          user: userAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([userAccount])
        .rpc();

      const balanceBefore = await provider.connection.getBalance(
        userAccount.publicKey
      );

      await program.methods
        .closeProfile()
        .accounts({
          profile: profilePda,
          user: userAccount.publicKey,
        })
        .signers([userAccount])
        .rpc();

      const balanceAfter = await provider.connection.getBalance(
        userAccount.publicKey
      );

      // Should have more balance (rent returned minus tx fee)
      expect(balanceAfter).to.be.greaterThan(balanceBefore - 10000);

      // Account should be closed
      try {
        await program.account.profile.fetch(profilePda);
        expect.fail("Account should be closed");
      } catch (err) {
        expect(err.message).to.include("Account does not exist");
      }
    });
  });
});
```

## Integration Testing

### Cross-Pallet Integration (Substrate)
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_token_transfer_with_profile() {
        new_test_ext().execute_with(|| {
            let sender = 1;
            let receiver = 2;
            let amount = 100;

            // Create profiles for both
            assert_ok!(Profiles::create_profile(
                RuntimeOrigin::signed(sender),
                b"Sender".to_vec()
            ));
            assert_ok!(Profiles::create_profile(
                RuntimeOrigin::signed(receiver),
                b"Receiver".to_vec()
            ));

            // Transfer tokens
            assert_ok!(Balances::transfer(
                RuntimeOrigin::signed(sender),
                receiver.into(),
                amount
            ));

            // Verify balances updated correctly
            assert_eq!(
                Balances::free_balance(receiver),
                2000 + amount - MinDeposit::get()
            );
        });
    }

    #[test]
    fn test_full_lifecycle() {
        new_test_ext().execute_with(|| {
            let user = 1;

            // 1. Create profile
            assert_ok!(Profiles::create_profile(
                RuntimeOrigin::signed(user),
                b"User".to_vec()
            ));

            // 2. Update profile
            assert_ok!(Profiles::update_profile(
                RuntimeOrigin::signed(user),
                b"Updated".to_vec()
            ));

            // 3. Perform operations requiring profile
            assert_ok!(SomeOtherPallet::do_something(
                RuntimeOrigin::signed(user)
            ));

            // 4. Close profile
            assert_ok!(Profiles::close_profile(
                RuntimeOrigin::signed(user)
            ));

            // 5. Verify cleanup
            assert!(Profiles::get(user).is_none());
        });
    }
}
```

### End-to-End Testing (Solana)
```typescript
describe("E2E: Token Marketplace", () => {
  it("complete buy/sell flow", async () => {
    // Setup
    const seller = Keypair.generate();
    const buyer = Keypair.generate();
    const mint = await createMint(provider.connection, seller);
    
    // 1. Seller lists item
    const listingPda = await program.methods
      .listItem(new BN(1000), "NFT Name")
      .accounts({
        listing: listingPda,
        seller: seller.publicKey,
        mint: mint,
      })
      .signers([seller])
      .rpc();
    
    // 2. Buyer purchases
    await program.methods
      .purchase()
      .accounts({
        listing: listingPda,
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        mint: mint,
      })
      .signers([buyer])
      .rpc();
    
    // 3. Verify state
    const listing = await program.account.listing.fetch(listingPda);
    expect(listing.status).to.equal({ sold: {} });
    
    // 4. Verify ownership transfer
    const buyerTokenAccount = await getAssociatedTokenAddress(
      mint,
      buyer.publicKey
    );
    const balance = await provider.connection.getTokenAccountBalance(
      buyerTokenAccount
    );
    expect(balance.value.amount).to.equal("1");
  });
});
```

## Property-Based Testing

### Using Proptest (Substrate)
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn name_length_always_within_bounds(
            name in prop::collection::vec(any::<u8>(), 0..100)
        ) {
            new_test_ext().execute_with(|| {
                let account = 1;
                let result = MyPallet::create_profile(
                    RuntimeOrigin::signed(account),
                    name.clone()
                );

                if name.len() <= MaxNameLength::get() as usize && !name.is_empty() {
                    assert_ok!(result);
                } else {
                    assert!(result.is_err());
                }
            });
        }

        #[test]
        fn balance_never_negative(
            operations in prop::collection::vec(
                (1u64..100u64, 1u64..100u64),
                1..50
            )
        ) {
            new_test_ext().execute_with(|| {
                for (account, amount) in operations {
                    let balance = Balances::free_balance(account);
                    
                    if balance >= amount {
                        assert_ok!(Balances::transfer(
                            RuntimeOrigin::signed(account),
                            (account + 1).into(),
                            amount
                        ));
                    }

                    // Invariant: balance never negative
                    assert!(Balances::free_balance(account) >= 0);
                }
            });
        }

        #[test]
        fn total_supply_invariant(
            operations in prop::collection::vec(any::<u8>(), 0..100)
        ) {
            new_test_ext().execute_with(|| {
                let initial_supply = Balances::total_issuance();

                // Perform random operations
                for _ in operations {
                    // ... various operations
                }

                // Total supply should never change
                prop_assert_eq!(
                    Balances::total_issuance(),
                    initial_supply
                );
            });
        }
    }
}
```

## Fuzzing

### Cargo Fuzz Setup
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzzing
cargo fuzz init

# Create fuzz target
cargo fuzz add fuzz_instruction_parsing
```

### Fuzz Target
```rust
// fuzz/fuzz_targets/fuzz_instruction_parsing.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use my_program::instruction::MyInstruction;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as instruction
    if let Ok(instruction) = MyInstruction::try_from_slice(data) {
        // If parsing succeeds, ensure it doesn't panic
        match instruction {
            MyInstruction::Initialize { data } => {
                // Validate data
                assert!(data <= u64::MAX);
            }
            MyInstruction::Update { value } => {
                assert!(value > 0);
            }
            _ => {}
        }
    }
});

// Run fuzzer
// cargo fuzz run fuzz_instruction_parsing
```

## Load Testing

### Substrate Load Testing
```rust
#[test]
#[ignore] // Run separately with --ignored
fn load_test_concurrent_transactions() {
    new_test_ext().execute_with(|| {
        let num_accounts = 1000;
        let transactions_per_account = 10;

        for account in 1..=num_accounts {
            for _ in 0..transactions_per_account {
                assert_ok!(MyPallet::do_something(
                    RuntimeOrigin::signed(account)
                ));
            }
        }

        // Verify all transactions processed
        assert_eq!(TotalTransactions::<Test>::get(), num_accounts * transactions_per_account);
    });
}
```

### Solana Load Testing
```typescript
describe("Load Tests", () => {
  it("handles 1000 concurrent transactions", async () => {
    const users = Array.from({ length: 1000 }, () => Keypair.generate());
    
    // Airdrop to all users
    await Promise.all(
      users.map(user =>
        provider.connection.requestAirdrop(
          user.publicKey,
          anchor.web3.LAMPORTS_PER_SOL
        )
      )
    );
    
    // Execute transactions in parallel
    const txPromises = users.map(user =>
      program.methods
        .doSomething()
        .accounts({ user: user.publicKey })
        .signers([user])
        .rpc()
    );
    
    const results = await Promise.allSettled(txPromises);
    
    const successful = results.filter(r => r.status === "fulfilled").length;
    const failed = results.filter(r => r.status === "rejected").length;
    
    console.log(`Successful: ${successful}, Failed: ${failed}`);
    expect(successful).to.be.greaterThan(900); // 90% success rate
  });
});
```

## Security Testing

### Vulnerability Testing
```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_reentrancy_protection() {
        new_test_ext().execute_with(|| {
            // Attempt reentrant call
            // Should fail due to state being updated first
        });
    }

    #[test]
    fn test_integer_overflow_protection() {
        new_test_ext().execute_with(|| {
            let account = 1;
            
            // Try to cause overflow
            assert_err!(
                MyPallet::add_value(
                    RuntimeOrigin::signed(account),
                    u64::MAX
                ),
                Error::<Test>::Overflow
            );
        });
    }

    #[test]
    fn test_unauthorized_access() {
        new_test_ext().execute_with(|| {
            let owner = 1;
            let attacker = 2;
            
            // Owner creates resource
            assert_ok!(MyPallet::create(RuntimeOrigin::signed(owner)));
            
            // Attacker tries to modify
            assert_err!(
                MyPallet::update(RuntimeOrigin::signed(attacker)),
                Error::<Test>::Unauthorized
            );
        });
    }

    #[test]
    fn test_account_substitution() {
        new_test_ext().execute_with(|| {
            // Test that accounts can't be substituted in CPI calls
        });
    }
}
```

## Deployment Strategies

### Development → Testnet → Mainnet

#### 1. Development Network
```bash
# Substrate: Run local node
./target/release/node-template --dev --tmp

# Solana: Local validator
solana-test-validator \
  --reset \
  --ledger /tmp/test-ledger \
  --bpf-program YourProgramID program.so
```

#### 2. Testnet Deployment
```bash
# Substrate: Deploy to Rococo/Westend
# (Requires parachain slot or use standalone)

# Solana Devnet
solana config set --url devnet
anchor build
anchor deploy --provider.cluster devnet

# Verify deployment
solana program show <PROGRAM_ID>
```

#### 3. Mainnet Deployment Checklist
```
Pre-Deployment:
□ All tests passing (unit, integration, E2E)
□ Security audit completed
□ Code coverage >90%
□ Load testing passed
□ Documentation complete
□ Upgrade/emergency plan ready
□ Monitoring setup
□ Bug bounty program (optional)

Deployment:
□ Deploy to testnet first
□ Monitor for 1-2 weeks
□ Gradual rollout (if possible)
□ Monitor metrics closely
□ Have rollback plan ready

Post-Deployment:
□ Verify all functions work
□ Monitor for unusual activity
□ Track transaction patterns
□ Performance monitoring
□ User feedback collection
```

#### Upgrade Strategy
```rust
// Substrate: Forkless upgrade via sudo or governance
#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(T::DbWeight::get().writes(1))]
    pub fn set_code(origin: OriginFor<T>, code: Vec<u8>) -> DispatchResult {
        ensure_root(origin)?;
        frame_system::Pallet::<T>::set_code(RawOrigin::Root.into(), code)?;
        Ok(())
    }
}

// Solana: Deploy new program, migrate data if needed
// Use program-derived addresses for data accounts to maintain compatibility
```

## Monitoring and Maintenance

### Health Checks
```rust
// Implement health check endpoint
#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn health_check(origin: OriginFor<T>) -> DispatchResult {
        ensure_none(origin)?;
        
        // Check critical invariants
        let total_supply = TotalSupply::<T>::get();
        let sum_of_balances: BalanceOf<T> = Balances::<T>::iter()
            .map(|(_, balance)| balance)
            .fold(Zero::zero(), |acc, b| acc + b);
        
        ensure!(total_supply == sum_of_balances, Error::<T>::InvariantViolation);
        
        Self::deposit_event(Event::HealthCheckPassed);
        Ok(())
    }
}
```

### Metrics Collection
```rust
// Track important metrics
#[pallet::storage]
pub type Metrics<T: Config> = StorageValue<
    _,
    PalletMetrics,
    ValueQuery,
>;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo)]
pub struct PalletMetrics {
    pub total_transactions: u64,
    pub total_value_transferred: u128,
    pub active_users: u32,
    pub last_transaction_block: u32,
}

// Update metrics on each operation
impl<T: Config> Pallet<T> {
    fn update_metrics() {
        Metrics::<T>::mutate(|metrics| {
            metrics.total_transactions += 1;
            metrics.last_transaction_block = 
                <frame_system::Pallet<T>>::block_number();
        });
    }
}
```

### Alerting
```bash
# Setup monitoring with Prometheus + Grafana
# Create alerts for:
# - Unusual transaction volume
# - High error rates
# - Slow block times
# - Low validator participation
# - Storage exhaustion warnings
```

This comprehensive testing and deployment strategy ensures robust, production-ready blockchain applications.
