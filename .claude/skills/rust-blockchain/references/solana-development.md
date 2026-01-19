# Solana Development with Rust

## Table of Contents
- [Solana Overview](#solana-overview)
- [Program Architecture](#program-architecture)
- [Anchor Framework](#anchor-framework)
- [Account Model](#account-model)
- [Instructions & Transactions](#instructions--transactions)
- [Program Derived Addresses (PDAs)](#program-derived-addresses-pdas)
- [Cross-Program Invocation (CPI)](#cross-program-invocation-cpi)
- [Testing](#testing)

## Solana Overview

Solana is a high-performance blockchain supporting 50,000+ TPS with sub-second finality.

### Key Features
- **Proof of History (PoH)**: Cryptographic clock for ordering transactions
- **Parallel transaction processing**: Sealevel runtime
- **Low fees**: ~$0.00025 per transaction
- **Rust-native**: Programs written in Rust compile to BPF
- **Account model**: Programs are stateless, state stored in accounts

### Architecture
```
┌─────────────────────────┐
│     Client (RPC)        │
├─────────────────────────┤
│   Solana Runtime        │
│   ├── Program A         │
│   ├── Program B         │
│   └── System Programs   │
├─────────────────────────┤
│   Accounts (State)      │
└─────────────────────────┘
```

## Program Architecture

### Native Solana Program
```rust
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Declare program entrypoint
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello from my Solana program!");
    
    // Parse accounts
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    
    // Verify account ownership
    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    
    // Parse instruction
    let instruction = instruction_data
        .get(0)
        .ok_or(ProgramError::InvalidInstructionData)?;
    
    match instruction {
        0 => process_initialize(accounts, instruction_data)?,
        1 => process_update(accounts, instruction_data)?,
        _ => return Err(ProgramError::InvalidInstructionData),
    }
    
    Ok(())
}

fn process_initialize(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Implementation
    Ok(())
}

fn process_update(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Implementation
    Ok(())
}
```

### Program Structure
```
my-solana-program/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Entrypoint
│   ├── processor.rs        # Business logic
│   ├── instruction.rs      # Instruction definitions
│   ├── state.rs           # Account state structures
│   └── error.rs           # Custom errors
├── tests/
│   └── integration.rs
└── Anchor.toml            # If using Anchor
```

## Anchor Framework

Anchor is the recommended framework for Solana development, providing:
- Automatic serialization/deserialization
- Account validation
- Type-safe instruction builders
- Built-in security checks

### Complete Anchor Program
```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("YourProgramIDHere11111111111111111111111111");

#[program]
pub mod my_program {
    use super::*;

    /// Initialize a new account
    pub fn initialize(
        ctx: Context<Initialize>,
        data: u64,
        bump: u8,
    ) -> Result<()> {
        let account = &mut ctx.accounts.my_account;
        account.authority = ctx.accounts.authority.key();
        account.data = data;
        account.bump = bump;
        
        msg!("Account initialized with data: {}", data);
        Ok(())
    }

    /// Update existing account
    pub fn update(
        ctx: Context<Update>,
        new_data: u64,
    ) -> Result<()> {
        let account = &mut ctx.accounts.my_account;
        
        require!(
            ctx.accounts.authority.key() == account.authority,
            ErrorCode::Unauthorized
        );
        
        account.data = new_data;
        
        emit!(DataUpdated {
            authority: ctx.accounts.authority.key(),
            old_data: account.data,
            new_data,
        });
        
        Ok(())
    }

    /// Transfer tokens with program authority
    pub fn transfer_tokens(
        ctx: Context<TransferTokens>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    /// Close account and return rent
    pub fn close(ctx: Context<Close>) -> Result<()> {
        let account = &ctx.accounts.my_account;
        
        require!(
            ctx.accounts.authority.key() == account.authority,
            ErrorCode::Unauthorized
        );
        
        Ok(())
    }
}

// Context structs define account requirements

#[derive(Accounts)]
#[instruction(data: u64, bump: u8)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + MyAccount::INIT_SPACE,
        seeds = [b"my-account", authority.key().as_ref()],
        bump,
    )]
    pub my_account: Account<'info, MyAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(
        mut,
        seeds = [b"my-account", authority.key().as_ref()],
        bump = my_account.bump,
    )]
    pub my_account: Account<'info, MyAccount>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [b"my-account", authority.key().as_ref()],
        bump = my_account.bump,
    )]
    pub my_account: Account<'info, MyAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
}

// Account structures

#[account]
#[derive(InitSpace)]
pub struct MyAccount {
    pub authority: Pubkey,
    pub data: u64,
    pub bump: u8,
}

// Events

#[event]
pub struct DataUpdated {
    pub authority: Pubkey,
    pub old_data: u64,
    pub new_data: u64,
}

// Errors

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    
    #[msg("The provided data is invalid")]
    InvalidData,
    
    #[msg("Arithmetic overflow occurred")]
    Overflow,
}
```

## Account Model

### Account Structure
```rust
#[account]
#[derive(InitSpace)]
pub struct GameState {
    pub authority: Pubkey,        // 32 bytes
    pub players: Vec<Pubkey>,     // 4 + (32 * players.len())
    pub score: u64,               // 8 bytes
    pub active: bool,             // 1 byte
    pub created_at: i64,          // 8 bytes
    #[max_len(50)]
    pub name: String,             // 4 + name.len()
}

// Space calculation:
// Discriminator: 8 bytes (Anchor adds automatically)
// authority: 32
// players: 4 + (32 * max_players)
// score: 8
// active: 1
// created_at: 8
// name: 4 + 50
// Total: 8 + 32 + 4 + 8 + 1 + 8 + 54 = 115 + (32 * max_players)
```

### Zero-Copy Accounts
For large accounts (>10KB), use zero-copy:

```rust
use anchor_lang::prelude::*;

#[account(zero_copy)]
pub struct LargeAccount {
    pub data: [u8; 10000],
    pub counter: u64,
}

#[derive(Accounts)]
pub struct UpdateLarge<'info> {
    #[account(mut)]
    pub large_account: AccountLoader<'info, LargeAccount>,
    pub authority: Signer<'info>,
}

pub fn update_large(ctx: Context<UpdateLarge>, value: u8) -> Result<()> {
    let mut account = ctx.accounts.large_account.load_mut()?;
    account.data[0] = value;
    account.counter += 1;
    Ok(())
}
```

## Instructions & Transactions

### Instruction Data Serialization
```rust
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum GameInstruction {
    Initialize {
        max_players: u8,
        entry_fee: u64,
    },
    JoinGame {
        player_name: String,
    },
    Play {
        move_data: [u8; 32],
    },
    EndGame,
}

// Deserialize in processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = GameInstruction::try_from_slice(instruction_data)?;
    
    match instruction {
        GameInstruction::Initialize { max_players, entry_fee } => {
            process_initialize(accounts, max_players, entry_fee)
        }
        GameInstruction::JoinGame { player_name } => {
            process_join(accounts, player_name)
        }
        GameInstruction::Play { move_data } => {
            process_play(accounts, move_data)
        }
        GameInstruction::EndGame => {
            process_end(accounts)
        }
    }
}
```

### Account Validation
```rust
#[derive(Accounts)]
pub struct SecureOperation<'info> {
    // ✅ Must be a signer
    pub authority: Signer<'info>,
    
    // ✅ Must be mutable
    #[account(mut)]
    pub target_account: Account<'info, MyAccount>,
    
    // ✅ Verify ownership
    #[account(
        mut,
        has_one = authority,  // target_account.authority == authority.key()
    )]
    pub owned_account: Account<'info, MyAccount>,
    
    // ✅ Verify specific address
    #[account(address = EXPECTED_ADDRESS)]
    pub specific_account: Account<'info, MyAccount>,
    
    // ✅ Verify constraint
    #[account(
        constraint = user_account.data > 0 @ ErrorCode::InvalidData
    )]
    pub user_account: Account<'info, MyAccount>,
}
```

## Program Derived Addresses (PDAs)

PDAs are deterministic addresses derived from seeds:

```rust
// Finding PDA
pub fn find_game_pda(
    authority: &Pubkey,
    game_id: u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"game",
            authority.as_ref(),
            &game_id.to_le_bytes(),
        ],
        program_id,
    )
}

// Using PDAs in Anchor
#[derive(Accounts)]
#[instruction(game_id: u64)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GameState::INIT_SPACE,
        seeds = [b"game", authority.key().as_ref(), &game_id.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, GameState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// Signing with PDA
pub fn pda_transfer(ctx: Context<PdaTransfer>, amount: u64) -> Result<()> {
    let seeds = &[
        b"vault",
        ctx.accounts.authority.key().as_ref(),
        &[ctx.accounts.vault.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token.to_account_info(),
        to: ctx.accounts.destination.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::transfer(cpi_ctx, amount)?;
    Ok(())
}
```

## Cross-Program Invocation (CPI)

### Calling Another Program
```rust
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

pub fn transfer_spl_tokens(
    ctx: Context<TransferSpl>,
    amount: u64,
) -> Result<()> {
    // Setup CPI accounts
    let cpi_accounts = Transfer {
        from: ctx.accounts.from.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    // Create CPI context
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    // Invoke
    token::transfer(cpi_ctx, amount)?;
    
    Ok(())
}

// With PDA signer
pub fn transfer_with_pda(
    ctx: Context<TransferWithPda>,
    amount: u64,
    bump: u8,
) -> Result<()> {
    let seeds = &[
        b"authority",
        ctx.accounts.base.key().as_ref(),
        &[bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.from.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        authority: ctx.accounts.pda_authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::transfer(cpi_ctx, amount)?;
    
    Ok(())
}
```

### Calling Custom Programs
```rust
// Program A calling Program B
pub fn call_other_program(ctx: Context<CallOther>) -> Result<()> {
    // Build instruction
    let ix = Instruction {
        program_id: ctx.accounts.other_program.key(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.authority.key(), true),
        ],
        data: vec![0, 1, 2, 3],  // Instruction data
    };
    
    // Invoke
    invoke(
        &ix,
        &[
            ctx.accounts.account.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.other_program.to_account_info(),
        ],
    )?;
    
    Ok(())
}
```

## Testing

### Anchor Tests
```typescript
// tests/my-program.ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MyProgram } from "../target/types/my_program";
import { assert } from "chai";

describe("my-program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MyProgram as Program<MyProgram>;
  
  let myAccount: anchor.web3.Keypair;
  let bump: number;

  before(async () => {
    myAccount = anchor.web3.Keypair.generate();
    
    // Find PDA
    const [pda, _bump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("my-account"),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );
    myAccount = pda;
    bump = _bump;
  });

  it("Initializes account", async () => {
    const tx = await program.methods
      .initialize(new anchor.BN(42), bump)
      .accounts({
        myAccount: myAccount,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const account = await program.account.myAccount.fetch(myAccount);
    assert.equal(account.data.toNumber(), 42);
    assert.equal(account.authority.toString(), provider.wallet.publicKey.toString());
  });

  it("Updates account", async () => {
    await program.methods
      .update(new anchor.BN(100))
      .accounts({
        myAccount: myAccount,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const account = await program.account.myAccount.fetch(myAccount);
    assert.equal(account.data.toNumber(), 100);
  });

  it("Fails with unauthorized user", async () => {
    const otherUser = anchor.web3.Keypair.generate();
    
    try {
      await program.methods
        .update(new anchor.BN(200))
        .accounts({
          myAccount: myAccount,
          authority: otherUser.publicKey,
        })
        .signers([otherUser])
        .rpc();
      
      assert.fail("Should have thrown error");
    } catch (err) {
      assert.include(err.toString(), "Unauthorized");
    }
  });

  it("Closes account", async () => {
    await program.methods
      .close()
      .accounts({
        myAccount: myAccount,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    // Account should be closed
    try {
      await program.account.myAccount.fetch(myAccount);
      assert.fail("Account should be closed");
    } catch (err) {
      assert.include(err.toString(), "Account does not exist");
    }
  });
});
```

### Rust Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    use solana_program_test::*;
    use solana_sdk::{signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn test_initialize() {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "my_program",
            program_id,
            processor!(entry),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Test logic
    }
}
```

### Best Practices
```rust
// ✅ Always validate signers
#[account(signer)]
pub authority: Signer<'info>,

// ✅ Check account ownership
require!(
    account.owner == expected_owner,
    ErrorCode::InvalidOwner
);

// ✅ Use safe math
let result = value1.checked_add(value2).ok_or(ErrorCode::Overflow)?;

// ✅ Close accounts to reclaim rent
#[account(
    mut,
    close = authority,
)]
pub account_to_close: Account<'info, MyAccount>,

// ✅ Validate data before processing
require!(data.len() <= MAX_SIZE, ErrorCode::DataTooLarge);

// ❌ Never trust client data without validation
// ❌ Never perform unbounded loops
// ❌ Never skip signer checks
```

This covers professional Solana program development with Anchor framework.
