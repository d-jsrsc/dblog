use anchor_lang::prelude::*;
use solana_program::{
    account_info::AccountInfo,
    program_memory::sol_memcmp,
    program_pack::{IsInitialized, Pack},
    pubkey::{Pubkey, PUBKEY_BYTES},
};

use crate::errors::BlogError;

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<()> {
    if !cmp_pubkeys(account.owner, owner) {
        err!(BlogError::IncurrentOwner)
    } else {
        Ok(())
    }
}

pub fn puffed_out_string(s: &str, size: usize) -> String {
    let mut array_of_zeroes = vec![];
    let puff_amount = size - s.len();
    while array_of_zeroes.len() < puff_amount {
        array_of_zeroes.push(0u8);
    }
    s.to_owned() + std::str::from_utf8(&array_of_zeroes).unwrap()
}

// pub fn assert_initialized<T: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<T> {
//     let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
//     if !account.is_initialized() {
//         err!(BlogError::CollectionNotVerified)
//     } else {
//         Ok(account)
//     }
// }
