use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::error::AMMErrors::{InvalidInstructionData, InvalidInstructionMethodID};
use std::mem::size_of;

#[derive(Debug)]
pub enum AMMInstruction {
    /// Initialize AMM
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` Owner
    /// 1. `[writable]` AMM data account
    /// 2. `[writable]` Initial X token account
    /// 3. `[writable]` X token vault
    /// 4. `[]` X token mint
    /// 5. `[writable]` Initial Y token account
    /// 6. `[writable]` Y token vault
    /// 7. `[]` Y token mint
    /// 8. `[]` System program`
    /// 9. `[]` Rent sysvar`
    /// 10. `[]` Token program`
    Initialize,

    /// Exchange
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` User
    /// 1. `[writable]` AMM data account
    /// 2. `[writable]` First token temp account
    /// 3. `[writable]` Second token user account
    /// 4. `[writable]` X token vault
    /// 5. `[writable]` Y token vault
    /// 6. `[]` Token program`
    Exchange,
}

impl AMMInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, _) = input.split_first().ok_or(InvalidInstructionData)?;

        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::Exchange,
            _ => return Err(InvalidInstructionMethodID.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::Initialize => buf.push(0),
            Self::Exchange => buf.push(1),
        }
        buf
    }
}

/// Creates a `Initialize` instruction.
pub fn initialize_amm(
    owner_pubkey: &Pubkey,
    temp_x_token: &Pubkey,
    x_mint: &Pubkey,
    temp_y_token: &Pubkey,
    y_mint: &Pubkey,
    amm_program_id: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    let data = AMMInstruction::Initialize.pack();

    let (amm_data_account, _) = Pubkey::find_program_address(&[b"data"], amm_program_id);
    let (x_vault_address, _) = Pubkey::find_program_address(&[b"x_vault"], amm_program_id);
    let (y_vault_address, _) = Pubkey::find_program_address(&[b"y_vault"], amm_program_id);

    let accounts = vec![
        AccountMeta::new(*owner_pubkey, true),
        AccountMeta::new(amm_data_account, false),
        AccountMeta::new(*temp_x_token, false),
        AccountMeta::new(x_vault_address, false),
        AccountMeta::new_readonly(*x_mint, false),
        AccountMeta::new(*temp_y_token, false),
        AccountMeta::new(y_vault_address, false),
        AccountMeta::new_readonly(*y_mint, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(*token_program_id, false),
    ];

    Instruction {
        program_id: *amm_program_id,
        accounts,
        data,
    }
}

pub fn exchange(
    user_pubkey: &Pubkey,
    temp_first_token_account: &Pubkey,
    user_second_token_account: &Pubkey,
    token_program_id: &Pubkey,
    amm_program_id: &Pubkey,
) -> Instruction {
    let data = AMMInstruction::Exchange.pack();

    let (amm_data_account, _) = Pubkey::find_program_address(&[b"data"], amm_program_id);
    let (x_vault_address, _) = Pubkey::find_program_address(&[b"x_vault"], amm_program_id);
    let (y_vault_address, _) = Pubkey::find_program_address(&[b"y_vault"], amm_program_id);

    let accounts = vec![
        AccountMeta::new(*user_pubkey, true),
        AccountMeta::new(amm_data_account, false),
        AccountMeta::new(*temp_first_token_account, false),
        AccountMeta::new(*user_second_token_account, false),
        AccountMeta::new(x_vault_address, false),
        AccountMeta::new(y_vault_address, false),
        AccountMeta::new_readonly(*token_program_id, false),
    ];

    Instruction {
        program_id: *amm_program_id,
        accounts,
        data,
    }
}
