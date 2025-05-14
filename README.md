# Solana Multisig Wallet

A secure multisig wallet implementation for Solana blockchain using the Anchor framework. This program implements a wallet system inspired by Argent wallet functionality, requiring multiple signatures for transaction execution and providing robust security features.

## Features

- **Dual Control**: Requires both owner and guardian signatures for critical operations
- **Account Management**: Create and manage multisig accounts with owner and guardian
- **Transaction Execution**: Execute transactions only when approved by both owner and guardian
- **Key Rotation**: Change owner or guardian with appropriate security measures
- **Guardian Backup**: Add a backup guardian for additional security
- **Escape Mechanism**: Recovery system with configurable security period
  - Owner can initiate guardian escape (to replace a guardian)
  - Guardian can initiate owner escape (to replace an owner)
  - Security period ensures time for intervention if unauthorized
- **Program Upgrades**: Secure program upgrade functionality with dual control

## Architecture

### Account Structure

The program uses a PDA (Program Derived Address) to store the multisig account data:

```
ArgentAccount {
    owner: Pubkey,              // The owner's public key
    guardian: Pubkey,           // The guardian's public key
    guardian_backup: Option<Pubkey>,  // Optional backup guardian
    escape_type: EscapeType,    // Current escape status
    escape_initiated_at: i64,   // Timestamp when escape was initiated
    security_period: i64,       // Security period in seconds (default 7 days)
    pending_tx: Option<PendingTransaction>,  // Pending transaction data
}
```

### Security Model

The security model is based on dual control, requiring both owner and guardian signatures for critical operations. This provides protection against:

- Single key compromise
- Accidental key loss
- Unauthorized transactions

The escape mechanism provides a fallback recovery option with a time-delay security period, allowing for intervention in case of suspicious activity.

## Installation

### Prerequisites

- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://www.anchor-lang.com/docs/installation)
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) and [Yarn](https://yarnpkg.com/)

### Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/multisig.git
   cd multisig
   ```

2. Install dependencies:

   ```bash
   yarn install
   ```

3. Build the program:

   ```bash
   anchor build
   ```

4. Deploy to localnet:
   ```bash
   anchor deploy
   ```

## Usage

### Creating a Multisig Account

```typescript
const owner = Keypair.generate();
const guardian = Keypair.generate();

// Create the multisig account
await program.methods
  .create(
    owner.publicKey,
    guardian.publicKey,
    new anchor.BN(604800) // 7 days security period
  )
  .accounts({
    payer: wallet.publicKey,
  })
  .rpc();
```

### Executing a Transaction

```typescript
const txData = Buffer.from("transaction data");

// Execute with both signatures
await program.methods
  .execute(txData)
  .accounts({
    argentAccount: argentAccountPda,
    owner: owner.publicKey,
    guardian: guardian.publicKey,
  })
  .signers([owner, guardian])
  .rpc();
```

### Changing Owner

```typescript
const newOwner = Keypair.generate();
const newOwnerSignature = await signMessage(newOwner, "authorization");

await program.methods
  .changeOwner(newOwner.publicKey, newOwnerSignature)
  .accounts({
    argentAccount: argentAccountPda,
    owner: owner.publicKey,
    guardian: guardian.publicKey,
  })
  .signers([owner, guardian])
  .rpc();
```

### Initiating Guardian Escape

```typescript
// Owner initiates guardian escape
await program.methods
  .triggerEscapeGuardian()
  .accounts({
    argentAccount: argentAccountPda,
    owner: owner.publicKey,
  })
  .signers([owner])
  .rpc();

// After security period elapses
const newGuardian = Keypair.generate();
await program.methods
  .escapeGuardian(newGuardian.publicKey)
  .accounts({
    argentAccount: argentAccountPda,
    owner: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

## Testing

Run the test suite:

```bash
anchor test
```

The tests cover:

- Basic functionality (account creation, transaction execution)
- Owner and guardian management
- Escape mechanism
- Edge cases and failure scenarios

## Development

### Project Structure

```
multisig/
├── programs/
│   └── multisig/
│       ├── src/
│       │   └── lib.rs       # Main program code
│       └── Cargo.toml       # Rust dependencies
├── tests/
│   └── multisig.ts          # Test suite
├── migrations/
│   └── deploy.ts            # Deployment script
├── Anchor.toml              # Anchor configuration
└── package.json             # JavaScript dependencies
```

### Building

```bash
anchor build
```

### Linting

```bash
yarn lint
```

### Fix Linting Issues

```bash
yarn lint:fix
```

## License

ISC License
