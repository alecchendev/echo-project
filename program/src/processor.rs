use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program::{invoke, invoke_signed},
    sysvar::{rent::Rent, Sysvar},
    program_pack::Pack,
};

// accounts
use solana_program::account_info::next_account_info;

use crate::error::EchoError;
use crate::instruction::EchoInstruction;

use crate::state::AuthorizedBufferHeader;

use spl_token::{
    id, instruction,
    state::{Account as TokenAccount, Mint}
};

pub fn assert_with_msg(statement: bool, err: ProgramError, msg: &str) -> ProgramResult {
    if !statement {
        msg!(msg);
        Err(err)
    } else {
        Ok(())
    }
}

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EchoInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            EchoInstruction::Echo { data } => {
                msg!("Instruction: Echo");

                // Iterating accounts is safer then indexing
                let accounts_iter = &mut accounts.iter();

                // Get the account to write to
                let echo_buffer = next_account_info(accounts_iter)?;

                // check if echo_buffer got any non-zero data, fail
                if !echo_buffer.data.borrow().iter().all(|&x| x == 0) {
                    msg!("MY_ERR: non-zero data in echo_buffer");
                    return Err(EchoError::Nonzero.into())
                }

                // first 4 bytes are messageLen
                let len_arr: [u8; 4] = instruction_data[1..5].try_into().expect("");
                let echo_len: usize = u32::from_le_bytes(len_arr).try_into().expect("");

                // get min size echo_buffer vs data
                let buffer_len = echo_buffer.data.borrow().iter().count();
                assert_with_msg(buffer_len > 0, ProgramError::MissingRequiredSignature, "echo account not initialized")?;
                msg!("buffer_len: {}\techo_len: {}", buffer_len, echo_len);
                let size = if buffer_len < echo_len { buffer_len } else { echo_len };

                // put data in account echo_buffer
                let mut echo_data = echo_buffer.try_borrow_mut_data()?;
                for i in 0..size {
                    echo_data[i] = data[i];
                }

                Ok(())
            }
            EchoInstruction::InitializeAuthorizedEcho {
                buffer_seed,
                buffer_size
            } => {
                msg!("Instruction: InitializeAuthorizedEcho");
                msg!("buffer_seed: {}\tbuffer_size: {}", buffer_seed, buffer_size);
                // Err(EchoError::NotImplemented.into())

                // Iterating accounts is safer then indexing
                let accounts_iter = &mut accounts.iter();

                // Get accounts
                let authorized_buffer = next_account_info(accounts_iter)?;
                let authority = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;

                // ensure authority is signer
                assert_with_msg(
                    authority.is_signer,
                    ProgramError::MissingRequiredSignature,
                    "Authority must sign",
                )?;
                
                // // create key - already done
                let (authorized_buffer_key, bump) = Pubkey::find_program_address(
                    &[
                        b"authority",
                        authority.key.as_ref(),
                        &buffer_seed.to_le_bytes()
                    ],
                    program_id,
                );

                msg!("bump: {}", bump);

                assert_with_msg(
                    authorized_buffer_key == *authorized_buffer.key,
                    ProgramError::MissingRequiredSignature,
                    "Attempted to allocate with an invalid authority",
                )?;

                invoke_signed(
                    &system_instruction::create_account(
                        authority.key,
                        authorized_buffer.key,
                        Rent::get()?.minimum_balance(9 + buffer_size),
                        9 + (buffer_size as u64), // for header
                        program_id,
                    ),
                    &[authority.clone(), authorized_buffer.clone(), system_program.clone()],
                    &[&[b"authority", authority.key.as_ref(), &buffer_seed.to_le_bytes(), &[bump]]]
                )?;

                let mut buffer = authorized_buffer.try_borrow_mut_data()?;
                buffer[0] = bump;
                for i in 0..8 {
                    buffer[i + 1] = buffer_seed.to_le_bytes()[i];
                }

                Ok(())

            }
            EchoInstruction::AuthorizedEcho { data } => {
                msg!("Instruction: AuthorizedEcho");
                // Err(EchoError::NotImplemented.into())

                // Iterating accounts is safer then indexing
                let accounts_iter = &mut accounts.iter();

                // Get accounts
                let authorized_buffer = next_account_info(accounts_iter)?;
                let authority = next_account_info(accounts_iter)?;
                msg!("{}",authorized_buffer.owner);

                // Deserialize the first 9 bytes (very poorly..)
                let mut auth_buffer_data = authorized_buffer.try_borrow_mut_data()?;
                let bump = auth_buffer_data[0]; 
                let buffer_seed_arr: [u8; 8] = auth_buffer_data[1..9].try_into().expect("");
                let buffer_seed = u64::from_le_bytes(buffer_seed_arr);
                msg!("buffer_seed: {}", buffer_seed);

                // check authority is signer
                assert_with_msg(
                    authority.is_signer,
                    ProgramError::MissingRequiredSignature,
                    "Authority needs to be signer"
                )?;

                // check pda
                let authorized_buffer_key = Pubkey::create_program_address(
                    &[b"authority", authority.key.as_ref(), &buffer_seed.to_le_bytes(), &[bump]],
                    program_id)?;

                assert_with_msg(
                    authorized_buffer_key == *authorized_buffer.key,
                    ProgramError::MissingRequiredSignature,
                    "Attempted to echo with an invalid authority"
                )?;

                // get min length
                let echo_len = data.len();
                let buffer_len = auth_buffer_data.iter().count() - 9;
                let size = if buffer_len < echo_len { buffer_len } else { echo_len };

                // copy data
                for i in 0..size {
                    auth_buffer_data[9 + i] = data[i];
                }

                // zero out any remaining
                for i in size..buffer_len {
                    auth_buffer_data[9 + i] = 0;
                }


                Ok(())
            }
            EchoInstruction::InitializeVendingMachineEcho {
                price,
                buffer_size
            } => {
                msg!("Instruction: InitializeVendingMachineEcho");

                msg!("price: {}\tbuffer_size: {}", price, buffer_size);

                // get accounts
                let accounts_iter = &mut accounts.iter();

                let vending_machine_buffer = next_account_info(accounts_iter)?;
                let vending_machine_mint = next_account_info(accounts_iter)?;
                let payer = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;

                // ensure authority is signer
                // assert_with_msg(
                //     authority.is_signer,
                //     ProgramError::MissingRequiredSignature,
                //     "Authority must sign",
                // )?;
                
                // // create key - already done
                let (vending_machine_buffer_key, bump) = Pubkey::find_program_address(
                    &[
                        b"vending_machine",
                        vending_machine_mint.key.as_ref(),
                        &price.to_le_bytes()
                    ],
                    program_id,
                );

                msg!("bump: {}", bump);

                assert_with_msg(
                    vending_machine_buffer_key == *vending_machine_buffer.key,
                    ProgramError::MissingRequiredSignature,
                    "Attempted to allocate with an invalid authority",
                )?;

                invoke_signed(
                    &system_instruction::create_account(
                        payer.key,
                        vending_machine_buffer.key,
                        Rent::get()?.minimum_balance(9 + buffer_size),
                        9 + (buffer_size as u64), // for header
                        program_id,
                    ),
                    &[payer.clone(), vending_machine_buffer.clone(), system_program.clone()],
                    &[&[b"vending_machine", vending_machine_mint.key.as_ref(), &price.to_le_bytes(), &[bump]]]
                )?;

                let mut buffer = vending_machine_buffer.try_borrow_mut_data()?;
                buffer[0] = bump;
                for i in 0..8 {
                    buffer[i + 1] = price.to_le_bytes()[i];
                }


                // create vending_machine_buffer account - with owner EchoProgram

                // set the first 9 bytes: bump, price

                Ok(())
            }
            EchoInstruction::VendingMachineEcho { data } => {
                msg!("Instruction: VendingMachineEcho");
                // Err(EchoError::NotImplemented.into())
                
                // get accounts
                let accounts_iter = &mut accounts.iter();

                let vending_machine_buffer= next_account_info(accounts_iter)?;
                let user = next_account_info(accounts_iter)?;
                let user_token_account_ai = next_account_info(accounts_iter)?;
                let vending_machine_mint = next_account_info(accounts_iter)?;
                let token_program = next_account_info(accounts_iter)?;

                let user_token_account = TokenAccount::unpack(&user_token_account_ai.try_borrow_data()?)?;

                // Get buffer data
                let mut vending_machine_buffer_data = vending_machine_buffer.try_borrow_mut_data()?;
                let bump = vending_machine_buffer_data[0]; 
                let price_arr: [u8; 8] = vending_machine_buffer_data[1..9].try_into().expect("");
                let price = u64::from_le_bytes(price_arr);

                // check vending mint and user token account mint
                assert_with_msg(
                    user_token_account.mint == *vending_machine_mint.key,
                    ProgramError::MissingRequiredSignature,
                    "User token account mint doesn't match vending machine mint"
                )?;

                // check token account owner is user - CHECK ASSOCIATED???
                assert_with_msg(
                    user_token_account.owner == *user.key,
                    ProgramError::MissingRequiredSignature,
                    "User token account mint is not owned by user"
                )?;

                // check amount is great enough to burn
                assert_with_msg(
                    user_token_account.amount >= price,
                    ProgramError::MissingRequiredSignature,
                    "User token account mint is not owned by user"
                )?;

                // check pda
                let vending_machine_buffer_key = Pubkey::create_program_address(
                    &[
                        b"vending_machine",
                        vending_machine_mint.key.as_ref(),
                        &price.to_le_bytes(),
                        &[bump]
                    ],
                    program_id)?;

                assert_with_msg(
                    vending_machine_buffer_key == *vending_machine_buffer.key,
                    ProgramError::MissingRequiredSignature,
                    "Attempted to echo with an invalid authority"
                )?;

                // burn price tokens
                invoke(
                    &instruction::burn(
                        &spl_token::id(),
                        user_token_account_ai.key,
                        vending_machine_mint.key,
                        &user.key,
                        &[],
                        price).unwrap(),
                    &[user_token_account_ai.clone(), vending_machine_mint.clone(), user.clone()],
                )?;

                // write to echo
                // get min length
                let echo_len = data.len();
                let buffer_len = vending_machine_buffer_data.iter().count() - 9;
                let size = if buffer_len < echo_len { buffer_len } else { echo_len };

                // copy data
                for i in 0..size {
                    vending_machine_buffer_data[9 + i] = data[i];
                }

                // zero out any remaining
                for i in size..buffer_len {
                    vending_machine_buffer_data[9 + i] = 0;
                }


                Ok(())
            }
        }
    }
}
