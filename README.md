# Solana Token Launchpad

A lightweight Solana program (smart contract) built with Anchor for launching custom SPL tokens with a built-in protocol fee.

## Features

- **Token Creation**: Initialize new SPL tokens with custom name, symbol, decimals, and initial supply
- **Protocol Fee**: Built-in 0.5 SOL minimum fee for token creation (configurable)
- **Mint Authority**: PDA-based mint authority for secure additional minting
- **Additional Minting**: Creator can mint more tokens after launch
- **Transfer with Fee**: Trading transfers include 1% protocol fee automatically
- **Events**: Emits events for token creation, minting, and transfers for indexing

## Program Instructions

### `create_token`
Creates a new SPL token with protocol fee.

**Parameters:**
- `name`: Token name (max 32 chars)
- `symbol`: Token symbol (max 10 chars)
- `decimals`: Token decimals (max 18)
- `initial_supply`: Initial supply to mint
- `protocol_fee`: Fee in lamports (minimum 0.5 SOL = 500,000,000 lamports)

**Accounts:**
- `payer`: Signer, pays for account creation and protocol fee
- `mint`: New token mint (initialized)
- `token_account`: Associated token account for initial supply
- `mint_authority`: PDA used as mint/freeze authority
- `system_program`, `token_program`, `associated_token_program`, `rent`

### `mint_tokens`
Mint additional tokens to creator's associated token account.

**Parameters:**
- `amount`: Amount to mint

**Accounts:**
- `mint`: Token mint
- `token_account`: Creator's associated token account
- `mint_authority`: PDA mint authority
- `payer`: Creator signer
- `token_program`

### `transfer_with_fee`
Transfer tokens with 1% protocol fee.

**Parameters:**
- `amount`: Amount to transfer

**Accounts:**
- `from`: Sender's token account
- `to`: Recipient's token account
- `protocol_treasury`: Protocol fee collection account
- `authority`: Sender signer
- `token_program`

## Building

```bash
# Install Anchor (if not installed)
# cargo install --git https://github.com/anchor-lang/anchor-cli anchor-cli --locked

# Build
anchor build

# Run tests
anchor test
```

## Deployment

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet
anchor deploy --provider.cluster mainnet
```

## Tech Stack

- **Rust** — Systems programming language
- **Anchor Framework** — Solana smart contract framework
- **SPL Token** — Solana Program Library token standard
- **Solana SDK** — Core Solana development kit

## Testing

```bash
anchor test
```

## Example Usage

```bash
# Create a new token (via CLI or SDK)
# Token: "MyToken" (MTK), 9 decimals, 1M supply, 0.5 SOL fee
```

## Security Considerations

- Mint authority is a PDA (Program Derived Address) — no single private key controls minting
- Protocol fee enforced at token creation (minimum 0.5 SOL)
- All amounts validated > 0
- Decimals capped at 18 (SPL Token standard)

## License

MIT