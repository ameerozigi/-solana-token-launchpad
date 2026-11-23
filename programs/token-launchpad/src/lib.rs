//! Token Launchpad — Solana SPL Token Launcher with Protocol Fee
//!
//! Allows anyone to create and mint an SPL token with a built-in trading fee.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, InitializeMint, Transfer};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod token_launchpad {
    use super::*;

    /// Create a new SPL token with optional protocol fee
    /// 
    /// Accounts:
    /// 0. `[signer]` payer - pays for account creation and protocol fee
    /// 1. `[writable]` mint - the new token mint
    /// 2. `[writable]` token_account - associated token account for initial supply
    /// 3. `[]` mint_authority - PDA that will be mint authority
    /// 4. `[]` system_program
    /// 5. `[]` token_program
    /// 6. `[]` associated_token_program
    /// 7. `[]` rent
    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u64,
        protocol_fee: u64, // in lamports (SOL)
    ) -> Result<()> {
        // Validate inputs
        require!(name.len() <= 32, TokenLaunchpadError::NameTooLong);
        require!(symbol.len() <= 10, TokenLaunchpadError::SymbolTooLong);
        require!(decimals <= 18, TokenLaunchpadError::InvalidDecimals);
        require!(initial_supply > 0, TokenLaunchpadError::ZeroSupply);
        require!(protocol_fee >= 500_000_000, TokenLaunchpadError::FeeTooLow); // 0.5 SOL minimum

        let payer = &ctx.accounts.payer;
        let mint = &ctx.accounts.mint;
        let token_account = &ctx.accounts.token_account;
        let mint_authority = &ctx.accounts.mint_authority;
        let system_program = &ctx.accounts.system_program;
        let token_program = &ctx.accounts.token_program;
        let rent = &ctx.accounts.rent;

        // Transfer protocol fee from payer to protocol treasury
        let transfer_ix = anchor_lang::system_program::Transfer {
            from: payer.to_account_info(),
            to: mint_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(system_program.to_account_info(), transfer_ix);
        anchor_lang::system_program::transfer(cpi_ctx, protocol_fee)?;

        // Initialize mint
        let init_mint_ctx = CpiContext::new(
            token_program.to_account_info(),
            InitializeMint {
                mint: mint.to_account_info(),
                rent: rent.to_account_info(),
            },
        );
        token::initialize_mint(
            init_mint_ctx,
            decimals,
            &mint_authority.key(),
            Some(&mint_authority.key()),
        )?;

        // Mint initial supply to token account
        let mint_to_ctx = CpiContext::new_with_signer(
            token_program.to_account_info(),
            MintTo {
                mint: mint.to_account_info(),
                to: token_account.to_account_info(),
                authority: mint_authority.to_account_info(),
            },
            &[&[
                b"mint_authority",
                mint.key().as_ref(),
                &[ctx.bumps.mint_authority],
            ]],
        );
        token::mint_to(mint_to_ctx, initial_supply)?;

        // Emit event
        emit!(TokenCreated {
            mint: mint.key(),
            creator: payer.key(),
            name,
            symbol,
            decimals,
            initial_supply,
            protocol_fee,
        });

        Ok(())
    }

    /// Mint additional tokens (requires mint authority)
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, TokenLaunchpadError::ZeroAmount);

        let mint_authority = &ctx.accounts.mint_authority;
        let mint = &ctx.accounts.mint;
        let token_account = &ctx.accounts.token_account;
        let token_program = &ctx.accounts.token_program;

        let mint_to_ctx = CpiContext::new_with_signer(
            token_program.to_account_info(),
            MintTo {
                mint: mint.to_account_info(),
                to: token_account.to_account_info(),
                authority: mint_authority.to_account_info(),
            },
            &[&[
                b"mint_authority",
                mint.key().as_ref(),
                &[ctx.bumps.mint_authority],
            ]],
        );
        token::mint_to(mint_to_ctx, amount)?;

        emit!(TokensMinted {
            mint: mint.key(),
            amount,
        });

        Ok(())
    }

    /// Transfer tokens with protocol fee (for trading)
    pub fn transfer_with_fee(
        ctx: Context<TransferWithFee>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, TokenLaunchpadError::ZeroAmount);

        let from = &ctx.accounts.from;
        let to = &ctx.accounts.to;
        let authority = &ctx.accounts.authority;
        let protocol_treasury = &ctx.accounts.protocol_treasury;
        let token_program = &ctx.accounts.token_program;

        // Calculate fee (1% of amount)
        let fee = amount / 100;
        let net_amount = amount - fee;

        // Transfer net amount to recipient
        let transfer_ctx = CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: from.to_account_info(),
                to: to.to_account_info(),
                authority: authority.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, net_amount)?;

        // Transfer fee to protocol treasury
        let fee_ctx = CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: from.to_account_info(),
                to: protocol_treasury.to_account_info(),
                authority: authority.to_account_info(),
            },
        );
        token::transfer(fee_ctx, fee)?;

        emit!(TransferWithFeeEvent {
            from: from.key(),
            to: to.key(),
            amount,
            fee,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, decimals: u8, initial_supply: u64, protocol_fee: u64)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA used as mint authority
    #[account(
        init,
        payer = payer,
        space = 8,
        seeds = [b"mint_authority", mint.key().as_ref()],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA mint authority
    #[account(
        seeds = [b"mint_authority", mint.key().as_ref()],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferWithFee<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,

    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    #[account(mut)]
    pub protocol_treasury: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[event]
pub struct TokenCreated {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_supply: u64,
    pub protocol_fee: u64,
}

#[event]
pub struct TokensMinted {
    pub mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TransferWithFeeEvent {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub fee: u64,
}

#[error_code]
pub enum TokenLaunchpadError {
    #[msg("Token name too long (max 32 chars)")]
    NameTooLong,
    #[msg("Token symbol too long (max 10 chars)")]
    SymbolTooLong,
    #[msg("Decimals must be <= 18")]
    InvalidDecimals,
    #[msg("Initial supply must be > 0")]
    ZeroSupply,
    #[msg("Protocol fee too low (minimum 0.5 SOL)")]
    FeeTooLow,
    #[msg("Amount must be > 0")]
    ZeroAmount,
}