#![cfg(feature = "test-bpf")]

use {
    solana_sdk::{signature::{Signer, Keypair}, transaction::Transaction},
    solana_program_test::*,
    assert_matches::*,
    solana_program::{
        system_program,
        sysvar::{rent::Rent, Sysvar},
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_instruction,
    },
    solana_validator::test_validator::*,
};

#[test]
fn test_echo() {
    // let program_id = Pubkey::new_unique();
    // let mut program_test = ProgramTest::default();
    // program_test.add_program("echo", program_id, None);
    solana_logger::setup_with_default("solana_program_runtime=debug");
    let program_id = Pubkey::new_unique();
    let auth = Keypair::new();
    let data = "hello echo!";
    let echo_buffer = Keypair::new();
    let (test_validator, payer) = TestValidatorGenesis::default()
        .add_program("echo", program_id)
        // .add_account(
        //     auth.pubkey(),
        //     solana_sdk::account::Account {
        //         lamports: 100_000_000_000,
        //         data: vec![],
        //         owner: system_program::id(),
        //         ..solana_sdk::account::Account::default()
        //     },
        // )
        // .add_account(
        //     echo_buffer.pubkey(),
        //     solana_sdk::account::Account {
        //         lamports: Rent::default().minimum_balance(size),//0,
        //         data: Vec::from([0; size]),//vec![],
        //         owner: system_program::id(),
        //         ..solana_sdk::account::Account::default()
        //     },
        // )
        .start();
    let rpc_client = test_validator.get_rpc_client();

    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();

    // INSERT TESTS HERE

    let mut instr_data = vec![0];
    instr_data.append(&mut (data.len() as u32).to_le_bytes().to_vec());
    instr_data.append(&mut data.as_bytes().to_vec());
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &echo_buffer.pubkey(),
                Rent::default().minimum_balance(data.len()),
                data.len() as u64,
                &program_id,
            ),
            Instruction {
                program_id,
                accounts: vec![AccountMeta::new(echo_buffer.pubkey(), false)],
                data: instr_data,
            }
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&echo_buffer, &payer], recent_blockhash);

    assert_matches!(rpc_client.send_and_confirm_transaction(&transaction), Ok(_));

}
