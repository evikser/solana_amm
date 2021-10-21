use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

#[derive(Debug)]
pub struct AMM {
    pub is_initialized: bool,

    pub x_mint: Pubkey,
    pub x_amount: u64,

    pub y_mint: Pubkey,
    pub y_amount: u64,
}


impl Sealed for AMM {}

impl IsInitialized for AMM {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for AMM {
    const LEN: usize = 81;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, AMM::LEN];
        let (is_initialized, x_mint, x_amount, y_mint, y_amount) =
            array_refs![src, 1, 32, 8, 32, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        let x_mint = Pubkey::new(x_mint);
        let x_amount = u64::from_be_bytes(*x_amount);
        let y_mint = Pubkey::new(y_mint);
        let y_amount = u64::from_be_bytes(*y_amount);

        Ok(AMM {
            is_initialized,
            x_mint,
            x_amount,
            y_mint,
            y_amount,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, AMM::LEN];
        let (is_initialized_dst, x_mint_dst, x_amount_dst, y_mint_dst, y_amount_dst) =
            mut_array_refs![dst, 1, 32, 8, 32, 8];

        let AMM {
            is_initialized,
            x_mint,
            x_amount,
            y_mint,
            y_amount,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;

        x_mint_dst.copy_from_slice(&x_mint.to_bytes());
        x_amount_dst.copy_from_slice(&x_amount.to_be_bytes());
        y_mint_dst.copy_from_slice(&y_mint.to_bytes());
        y_amount_dst.copy_from_slice(&y_amount.to_be_bytes());
    }
}
