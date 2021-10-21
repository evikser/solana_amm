use rust_decimal::prelude::*;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{error::AMMErrors, instruction::AMMInstruction, state};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = AMMInstruction::unpack(instruction_data)?;

        match instruction {
            AMMInstruction::Initialize => Self::process_initialize(accounts, program_id),
            AMMInstruction::Exchange => Self::process_exchange(accounts, program_id),
        }
    }

    fn process_initialize(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let owner_account = next_account_info(accounts_iter)?;
        let amm_data_account = next_account_info(accounts_iter)?;
        let x_temp_account = next_account_info(accounts_iter)?;
        let x_vault_account = next_account_info(accounts_iter)?;
        let x_mint_account = next_account_info(accounts_iter)?;
        let y_temp_account = next_account_info(accounts_iter)?;
        let y_vault_account = next_account_info(accounts_iter)?;
        let y_mint_account = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;
        let rent_sysvar = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;

        let (amm_data_address, amm_data_bump_seed) =
            Pubkey::find_program_address(&[b"data"], program_id);

        if *amm_data_account.key != amm_data_address {
            return Err(AMMErrors::DataAccountMismatch.into());
        }

        if !amm_data_account.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let x_temp_data = spl_token::state::Account::unpack(&x_temp_account.data.borrow())?;
        let y_temp_data = spl_token::state::Account::unpack(&y_temp_account.data.borrow())?;

        if x_temp_data.mint != *x_mint_account.key
            || y_temp_data.mint != *y_mint_account.key
            || y_mint_account.key == x_mint_account.key
        {
            return Err(AMMErrors::TokenMintMismatch.into());
        }

        let rent = &Rent::from_account_info(rent_sysvar)?;
        let amm_data_signer_seeds: &[&[_]] = &[b"data", &[amm_data_bump_seed]];

        invoke_signed(
            &system_instruction::create_account(
                owner_account.key,
                amm_data_account.key,
                1.max(rent.minimum_balance(state::AMM::LEN)),
                state::AMM::LEN as u64,
                program_id,
            ),
            &[
                owner_account.clone(),
                amm_data_account.clone(),
                system_account.clone(),
            ],
            &[&amm_data_signer_seeds],
        )?;

        for (
            vault_seed,
            vault_account,
            mint_account,
            temp_token_account,
            temp_token_account_data,
        ) in [
            (
                b"x_vault".clone(),
                x_vault_account.clone(),
                x_mint_account.clone(),
                x_temp_account.clone(),
                x_temp_data,
            ),
            (
                b"y_vault".clone(),
                y_vault_account.clone(),
                y_mint_account.clone(),
                y_temp_account.clone(),
                y_temp_data,
            ),
        ]
        .iter()
        {
            let (vault_address, vault_bump_seed) =
                Pubkey::find_program_address(&[vault_seed], program_id);
            let vault_signer_seeds: &[&[_]] = &[vault_seed, &[vault_bump_seed]];

            invoke_signed(
                &system_instruction::create_account(
                    owner_account.key,
                    &vault_address,
                    1.max(rent.minimum_balance(spl_token::state::Account::LEN)),
                    spl_token::state::Account::LEN as u64,
                    token_program.key,
                ),
                &[vault_account.clone(), owner_account.clone()],
                &[&vault_signer_seeds],
            )?;

            invoke(
                &spl_token::instruction::initialize_account(
                    token_program.key,
                    vault_account.key,
                    mint_account.key,
                    vault_account.key,
                )?,
                &[
                    vault_account.clone(),
                    mint_account.clone(),
                    rent_sysvar.clone(),
                ],
            )?;

            invoke(
                &spl_token::instruction::transfer(
                    token_program.key,
                    &temp_token_account.key,
                    &vault_account.key,
                    &owner_account.key,
                    &[&owner_account.key],
                    temp_token_account_data.amount,
                )?,
                &[
                    temp_token_account.clone(),
                    owner_account.clone(),
                    vault_account.clone(),
                ],
            )?;
        }

        let amm_data = state::AMM {
            is_initialized: true,
            x_mint: x_temp_data.mint,
            x_amount: x_temp_data.amount,
            y_mint: y_temp_data.mint,
            y_amount: y_temp_data.amount,
        };

        state::AMM::pack(amm_data, &mut amm_data_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_exchange(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let user_account = next_account_info(accounts_iter)?;
        let amm_data_account = next_account_info(accounts_iter)?;
        let temp_first_token_account = next_account_info(accounts_iter)?;
        let user_second_token_account = next_account_info(accounts_iter)?;
        let x_token_vault = next_account_info(accounts_iter)?;
        let y_token_vault = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;

        let mut amm_data = state::AMM::unpack(&amm_data_account.data.borrow())?;
        let temp_first_token_account_data =
            spl_token::state::Account::unpack(&temp_first_token_account.data.borrow())?;
        let user_second_token_account_data =
            spl_token::state::Account::unpack(&user_second_token_account.data.borrow())?;

        let (first_token_vault, second_token_vault, second_token_vault_seed, second_token_amount) =
            if temp_first_token_account_data.mint == amm_data.x_mint
                && user_second_token_account_data.mint == amm_data.y_mint
            {
                let x_amount: Decimal = temp_first_token_account_data.amount.into();
                let current_x: Decimal = amm_data.x_amount.into();
                let current_y: Decimal = amm_data.y_amount.into();

                let new_x = current_x + x_amount;
                let new_y = current_y * current_x / new_x;

                let y_amount = current_y - new_y;

                amm_data.x_amount = new_x.round().trunc().mantissa() as u64;
                amm_data.y_amount = new_y.round().trunc().mantissa() as u64;

                (
                    x_token_vault.clone(),
                    y_token_vault.clone(),
                    b"y_vault".clone(),
                    y_amount.round().trunc().mantissa() as u64,
                )
            } else if temp_first_token_account_data.mint == amm_data.y_mint
                && user_second_token_account_data.mint == amm_data.x_mint
            {
                let y_amount: Decimal = temp_first_token_account_data.amount.into();
                let current_x: Decimal = amm_data.x_amount.into();
                let current_y: Decimal = amm_data.y_amount.into();

                let new_y = current_y + y_amount;
                let new_x = current_x * current_y / new_y;

                let x_amount = current_x - new_x;

                amm_data.x_amount = new_x.round().trunc().mantissa() as u64;
                amm_data.y_amount = new_y.round().trunc().mantissa() as u64;

                (
                    y_token_vault.clone(),
                    x_token_vault.clone(),
                    b"x_vault".clone(),
                    x_amount.round().trunc().mantissa() as u64,
                )
            } else {
                return Err(AMMErrors::TokenMintMismatch.into());
            };

        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                &temp_first_token_account.key,
                &first_token_vault.key,
                &user_account.key,
                &[&user_account.key],
                temp_first_token_account_data.amount,
            )?,
            &[
                temp_first_token_account.clone(),
                user_account.clone(),
                first_token_vault.clone(),
            ],
        )?;

        let (_, second_token_vault_bump_seed) =
            Pubkey::find_program_address(&[&second_token_vault_seed], program_id);
        let second_token_vault_signer_seeds: &[&[_]] =
            &[&second_token_vault_seed, &[second_token_vault_bump_seed]];

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                &second_token_vault.key,
                &user_second_token_account.key,
                &second_token_vault.key,
                &[&second_token_vault.key],
                second_token_amount,
            )?,
            &[
                user_second_token_account.clone(),
                second_token_vault.clone(),
            ],
            &[&second_token_vault_signer_seeds],
        )?;

        state::AMM::pack(amm_data, &mut amm_data_account.data.borrow_mut())?;

        Ok(())
    }
}
