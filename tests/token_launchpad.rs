use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport::TransportError,
};
use std::str::FromStr;

#[tokio::test]
async fn test_create_token() -> Result<(), TransportError> {
    let program_id = Pubkey::from_str("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS").unwrap();
    
    let mut test = ProgramTest::new(
        "token_launchpad",
        program_id,
        processor!(token_launchpad::entry),
    );
    test.add_program("anchor_spl_token", anchor_spl::token::ID, None);
    test.add_program("anchor_spl_associated_token", anchor_spl::associated_token::ID, None);

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    let mint = Keypair::new();
    let name = "Test Token".to_string();
    let symbol = "TEST".to_string();
    let decimals = 9;
    let initial_supply = 1_000_000_000; // 1M tokens
    let protocol_fee = 500_000_000; // 0.5 SOL

    let mint_authority = Pubkey::find_program_address(
        &[b"mint_authority", mint.pubkey().as_ref()],
        &program_id,
    ).0;

    let token_account = anchor_spl::associated_token::get_associated_token_address(
        &payer.pubkey(),
        &mint.pubkey(),
    );

    let ix = token_launchpad::instruction::CreateToken {
        payer: payer.pubkey(),
        mint: mint.pubkey(),
        token_account,
        mint_authority,
        system_program: system_program::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::ID,
    }
    .data(
        token_launchpad::instruction::CreateTokenInstructionData {
            name,
            symbol,
            decimals,
            initial_supply,
            protocol_fee,
        }
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await?;

    // Verify mint was created
    let mint_account = banks_client.get_account(mint.pubkey()).await?;
    assert!(mint_account.is_some());

    // Verify token account has initial supply
    let token_account_data = banks_client.get_account(token_account).await?;
    assert!(token_account_data.is_some());

    println!("Token created successfully!");
    Ok(())
}

#[tokio::test]
async fn test_mint_tokens() -> Result<(), TransportError> {
    let program_id = Pubkey::from_str("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS").unwrap();
    
    let mut test = ProgramTest::new(
        "token_launchpad",
        program_id,
        processor!(token_launchpad::entry),
    );
    test.add_program("anchor_spl_token", anchor_spl::token::ID, None);
    test.add_program("anchor_spl_associated_token", anchor_spl::associated_token::ID, None);

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    // First create a token
    let mint = Keypair::new();
    let name = "Test Token".to_string();
    let symbol = "TEST".to_string();
    let decimals = 9;
    let initial_supply = 1_000_000_000;
    let protocol_fee = 500_000_000;

    let mint_authority = Pubkey::find_program_address(
        &[b"mint_authority", mint.pubkey().as_ref()],
        &program_id,
    ).0;

    let token_account = anchor_spl::associated_token::get_associated_token_address(
        &payer.pubkey(),
        &mint.pubkey(),
    );

    let create_ix = token_launchpad::instruction::CreateToken {
        payer: payer.pubkey(),
        mint: mint.pubkey(),
        token_account,
        mint_authority,
        system_program: system_program::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::ID,
    }
    .data(
        token_launchpad::instruction::CreateTokenInstructionData {
            name,
            symbol,
            decimals,
            initial_supply,
            protocol_fee,
        }
    );

    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );

    banks_client.process_transaction(create_tx).await?;

    // Now mint additional tokens
    let additional_amount = 500_000_000; // 500M more tokens

    let mint_ix = token_launchpad::instruction::MintTokens {
        mint: mint.pubkey(),
        token_account,
        mint_authority,
        payer: payer.pubkey(),
        token_program: anchor_spl::token::ID,
    }
    .data(
        token_launchpad::instruction::MintTokensInstructionData {
            amount: additional_amount,
        }
    );

    let mint_tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(mint_tx).await?;

    // Verify total supply
    let token_account_data = banks_client.get_account(token_account).await?;
    assert!(token_account_data.is_some());

    println!("Additional tokens minted successfully!");
    Ok(())
}

#[tokio::test]
async fn test_transfer_with_fee() -> Result<(), TransportError> {
    let program_id = Pubkey::from_str("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS").unwrap();
    
    let mut test = ProgramTest::new(
        "token_launchpad",
        program_id,
        processor!(token_launchpad::entry),
    );
    test.add_program("anchor_spl_token", anchor_spl::token::ID, None);
    test.add_program("anchor_spl_associated_token", anchor_spl::associated_token::ID, None);

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    // Create token
    let mint = Keypair::new();
    let name = "Test Token".to_string();
    let symbol = "TEST".to_string();
    let decimals = 9;
    let initial_supply = 1_000_000_000;
    let protocol_fee = 500_000_000;

    let mint_authority = Pubkey::find_program_address(
        &[b"mint_authority", mint.pubkey().as_ref()],
        &program_id,
    ).0;

    let from_token_account = anchor_spl::associated_token::get_associated_token_address(
        &payer.pubkey(),
        &mint.pubkey(),
    );

    let recipient = Keypair::new();
    let to_token_account = anchor_spl::associated_token::get_associated_token_address(
        &recipient.pubkey(),
        &mint.pubkey(),
    );

    let protocol_treasury = Keypair::new();
    let protocol_treasury_account = anchor_spl::associated_token::get_associated_token_address(
        &protocol_treasury.pubkey(),
        &mint.pubkey(),
    );

    // Create token with initial supply
    let create_ix = token_launchpad::instruction::CreateToken {
        payer: payer.pubkey(),
        mint: mint.pubkey(),
        token_account: from_token_account,
        mint_authority,
        system_program: system_program::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::ID,
    }
    .data(
        token_launchpad::instruction::CreateTokenInstructionData {
            name,
            symbol,
            decimals,
            initial_supply,
            protocol_fee,
        }
    );

    let create_tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );

    banks_client.process_transaction(create_tx).await?;

    // Transfer with fee
    let transfer_amount = 100_000_000; // 100 tokens

    let transfer_ix = token_launchpad::instruction::TransferWithFee {
        from: from_token_account,
        to: to_token_account,
        protocol_treasury: protocol_treasury_account,
        authority: payer.pubkey(),
        token_program: anchor_spl::token::ID,
    }
    .data(
        token_launchpad::instruction::TransferWithFeeInstructionData {
            amount: transfer_amount,
        }
    );

    let transfer_tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transfer_tx).await?;

    println!("Transfer with fee executed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_validation_errors() -> Result<(), TransportError> {
    let program_id = Pubkey::from_str("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS").unwrap();
    
    let mut test = ProgramTest::new(
        "token_launchpad",
        program_id,
        processor!(token_launchpad::entry),
    );
    test.add_program("anchor_spl_token", anchor_spl::token::ID, None);
    test.add_program("anchor_spl_associated_token", anchor_spl::associated_token::ID, None);

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    // Test name too long
    let mint = Keypair::new();
    let long_name = "A".repeat(33);
    let mint_authority = Pubkey::find_program_address(
        &[b"mint_authority", mint.pubkey().as_ref()],
        &program_id,
    ).0;
    let token_account = anchor_spl::associated_token::get_associated_token_address(
        &payer.pubkey(),
        &mint.pubkey(),
    );

    let ix = token_launchpad::instruction::CreateToken {
        payer: payer.pubkey(),
        mint: mint.pubkey(),
        token_account,
        mint_authority,
        system_program: system_program::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::ID,
    }
    .data(
        token_launchpad::instruction::CreateTokenInstructionData {
            name: long_name,
            symbol: "TEST".to_string(),
            decimals: 9,
            initial_supply: 1_000_000_000,
            protocol_fee: 500_000_000,
        }
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(tx).await;
    assert!(result.is_err());

    // Test fee too low
    let mint2 = Keypair::new();
    let mint_authority2 = Pubkey::find_program_address(
        &[b"mint_authority", mint2.pubkey().as_ref()],
        &program_id,
    ).0;
    let token_account2 = anchor_spl::associated_token::get_associated_token_address(
        &payer.pubkey(),
        &mint2.pubkey(),
    );

    let ix2 = token_launchpad::instruction::CreateToken {
        payer: payer.pubkey(),
        mint: mint2.pubkey(),
        token_account: token_account2,
        mint_authority: mint_authority2,
        system_program: system_program::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::ID,
    }
    .data(
        token_launchpad::instruction::CreateTokenInstructionData {
            name: "Valid Name".to_string(),
            symbol: "VALID".to_string(),
            decimals: 9,
            initial_supply: 1_000_000_000,
            protocol_fee: 100_000_000, // Too low (0.1 SOL)
        }
    );

    let tx2 = Transaction::new_signed_with_payer(
        &[ix2],
        Some(&payer.pubkey()),
        &[&payer, &mint2],
        recent_blockhash,
    );

    let result2 = banks_client.process_transaction(tx2).await;
    assert!(result2.is_err());

    println!("Validation errors caught correctly!");
    Ok(())
}