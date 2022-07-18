mod errors;
mod utils;

use byteorder::{ByteOrder, LittleEndian};
use uuid::{uuid, Uuid};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use errors::BlogError;
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

declare_id!("4TWxkK23JiJGamgeUw4iGCf5KfRXCkobCaNxLhFXj1Ms");

const PUBKEY_LEN: usize = 32;
const ARWEAVE_KEY_LEN: usize = 43;
const NONCE_LEN: usize = 12;
const TITLE_LEN: usize = 80;
const UUID_LEN: usize = 36;
const ENCRYPT_WORD_LEN: usize =  6;

#[program]
pub mod dblog {
    use anchor_lang::system_program;

    use crate::utils::{assert_owned_by, cmp_pubkeys};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, nonce: String, arweave_key: String, title: String, encrypted: bool, encrypt_word: Option<String>) -> Result<()> {
        let now_ts = Clock::get().unwrap().unix_timestamp;
        let uuid_str = "dblog.xyz/nonce/".to_string() + nonce.as_str();

        ctx.accounts.blog.arweave_key = arweave_key;
        ctx.accounts.blog.nonce = nonce;
        ctx.accounts.blog.encrypted = encrypted;
        ctx.accounts.blog.created_time = now_ts;
        ctx.accounts.blog.title = title;
        ctx.accounts.blog.encrypt_word = encrypt_word;

        let owner = &ctx.accounts.owner.to_account_info();
        assert_owned_by(owner, &system_program::ID)?;

        ctx.accounts.blog.owner = ctx.accounts.owner.key();

        if ctx.remaining_accounts.is_empty() {
            let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, uuid_str.as_bytes());
            let id_str = id.to_string();
            ctx.accounts.blog.uuid = Some(id_str);
        } else {
            let mut remaining_accounts_counter: usize = 0;

            let pre_blog_info = &ctx.remaining_accounts[remaining_accounts_counter];
            assert_owned_by(pre_blog_info, &crate::id())?;

            let pre_blog: Blog = Blog::from_account_info(pre_blog_info)?;
            if !cmp_pubkeys(&pre_blog.owner, ctx.accounts.owner.key) {
                msg!("pre_blog {:?} {:?}", pre_blog.owner.to_string(), ctx.accounts.owner.key.to_string());
                return err!(BlogError::PreOwnerIsNotCurrentOwner);
            }
            ctx.accounts.blog.prev_blog = Some(pre_blog_info.key());
            remaining_accounts_counter += 1;

            if ctx.remaining_accounts.len() > remaining_accounts_counter {
                let _tag_info = &ctx.remaining_accounts[remaining_accounts_counter];
            }

            
        }

        Ok(())
    }
}

#[account]
pub struct Blog {
    owner: Pubkey,
    arweave_key: String,
    nonce: String,
    encrypted: bool,
    prev_blog: Option<Pubkey>,
    created_time: i64,
    title: String,
    tag: Option<Pubkey>,
    uuid: Option<String>,
    encrypt_word: Option<String>,
}

impl Blog {
    pub fn space() -> usize {
        8 +                         // anchor pre
        PUBKEY_LEN +                // owner
        4 + ARWEAVE_KEY_LEN +       // key
        4 + NONCE_LEN +             // nonce
        1 +                         // encrypted
        1 + PUBKEY_LEN +            // pre_blog
        8 +                         // created_time
        4 + TITLE_LEN +             // title
        1 + PUBKEY_LEN +            // tag
        1 + 4 + UUID_LEN +          // uuid
        1 + 4 + ENCRYPT_WORD_LEN    // encrypt_word
    }

    pub fn from_account_info(a: &AccountInfo) -> Result<Self> {
        let data = a.data.borrow();
        let data = data.as_ref();
        let mut cursor = 8;

        let owner = &data[cursor..cursor + PUBKEY_LEN];
        cursor += PUBKEY_LEN;
        
        let owner = match Pubkey::try_from_slice(owner) {
            Ok(owner) => owner,
            Err(_) => return err!(BlogError::FromKeyErr)
        };

        let key_pre = &data[cursor..cursor + 4];
        cursor += 4;
    
        let len = LittleEndian::read_uint(key_pre, 4) as usize;
        if len != ARWEAVE_KEY_LEN {
            return err!(BlogError::FromKeyErr) // TODO
        }
        let arweave_key = &data[cursor..cursor + ARWEAVE_KEY_LEN];
        cursor += ARWEAVE_KEY_LEN;
        let arweave_key = String::from_utf8_lossy(arweave_key).to_string();
    
        let nonce_pre = &data[cursor..cursor + 4];
        cursor += 4;
        let nonce_len = LittleEndian::read_uint(nonce_pre, 4) as usize;
        if nonce_len != NONCE_LEN {
            return err!(BlogError::FromKeyErr) // TODO
        }
        let nonce = &data[cursor..cursor + NONCE_LEN];
        cursor += NONCE_LEN;
        let nonce = String::from_utf8_lossy(nonce).to_string();

        let encrypted = &data[cursor..cursor + 1];
        cursor += 1;
        let encrypted = match *encrypted {
            [0] => false,
            [1] => true,
            _ => return err!(BlogError::FromNoneErr),
        };
    
        let mut prev_blog= None;
        let prev_blog_pre = &data[cursor..cursor + 1];
        cursor += 1;
        match *prev_blog_pre {
            [0] => {}
            [1] => {
                let prev_blog_buff = &data[cursor..cursor + PUBKEY_LEN];
                cursor += PUBKEY_LEN;
                prev_blog = match Pubkey::try_from_slice(prev_blog_buff) {
                    Ok(blog) => Some(blog),
                    Err(_) => return err!(BlogError::FromKeyErr) //TODO
                };
            }
            _ => return err!(BlogError::FromTitleErr) // TODO
        };
    
        let created_time = &data[cursor..cursor + 8];
        cursor += 8;
        let created_time = match i64::try_from_slice(created_time) {
            Ok(time) => time,
            Err(_) => return err!(BlogError::FromTitleErr) // TODO
        };
    
        let title_pre = &data[cursor..cursor + 4];
        cursor += 4;
        let title_len = LittleEndian::read_uint(title_pre, 4) as usize;

        let title = &data[cursor..cursor + title_len];
        cursor += title_len;
        let title = String::from_utf8_lossy(title).to_string();

        let mut tag = None;
        let tag_pre = &data[cursor..cursor + 1];
        cursor += 1;
        match *tag_pre {
            [0] => {}
            [1] => {
                let tag_buff = &data[cursor..cursor + PUBKEY_LEN];
                cursor += PUBKEY_LEN;
                tag = match Pubkey::try_from_slice(tag_buff) {
                    Ok(tag) => Some(tag),
                    Err(_) => return err!(BlogError::FromKeyErr) //TODO
                };
            }
            _ => return err!(BlogError::FromTitleErr) // TODO
        };
    
        let mut uuid = None;
        let uuid_pre = &data[cursor..cursor + 1];
        cursor += 1;
        match *uuid_pre {
            [0] => {}
            [1] => {
                let uuid_len = &data[cursor..cursor + 4];
                cursor += 4;
                let uuid_len = LittleEndian::read_uint(uuid_len, 4) as usize;
                if uuid_len != UUID_LEN {
                    return err!(BlogError::FromTitleErr); // TODO
                }
                // println!("uuid_len {:?}", uuid_len);
                let uuid_buff = &data[cursor..cursor + UUID_LEN];
                cursor+=UUID_LEN;
                uuid = Some(String::from_utf8_lossy(uuid_buff).to_string());
            }
            _ => return err!(BlogError::FromTitleErr) // TODO
        };

        let mut encrypt_word = None;
        let encrypt_word_exist = &data[cursor..cursor + 1];
        cursor += 1;
        match *encrypt_word_exist {
            [0] => {}
            [1] => {
                let encrypt_word_len = &data[cursor..cursor + 4];
                cursor += 4;
                let encrypt_word_len = LittleEndian::read_uint(encrypt_word_len, 4) as usize;
                if encrypt_word_len != ENCRYPT_WORD_LEN {
                    return err!(BlogError::FromTitleErr); // TODO
                }

                let encrypt_word_buff = &data[cursor..cursor + ENCRYPT_WORD_LEN];
                cursor += ENCRYPT_WORD_LEN;
                encrypt_word = Some(String::from_utf8_lossy(encrypt_word_buff).to_string());
            }
            _ => return err!(BlogError::FromTitleErr) // TODO
        };
    
        msg!("cursor: {:?}", cursor);
         Ok(Blog {
            owner, arweave_key, nonce, encrypted, prev_blog, created_time, title, tag, uuid, encrypt_word
        })
        
        // let md: Self = meta_deser(&mut a.data.borrow_mut().as_ref())?;
        // let data = a.data.borrow_mut();
        // let blog_data = &mut data.as_ref();
        // let src = array_ref![blog_data, 0, 299];
        // let (
        //     _anchor_pre, 
        //     owner, 
        //     _key_pre, key,
        //     _nonce_pre, nonce, 
        //     encrypted,
        //     pre_option,
        //     pre_blog,
        //     created_time,
        //     _title_pre,
        //     title,
        //     pre_tag,
        //     tag,
        //     pre_uuid,
        //     uuid
        // ) = 
        //     array_refs![src, 8, 32, 4 ,43, 4, 12, 1, 1, 32, 8, 4, 80, 1, 32, 1, 36];

        // let owner = Pubkey::new_from_array(*owner);
        // let key = match String::from_utf8(key.to_vec()) {
        //     Ok(k) => k,
        //     Err(_) => return err!(BlogError::FromKeyErr)
        // };
        // let nonce = match String::from_utf8(nonce.to_vec()) {
        //     Ok(k) => k,
        //     Err(_) => return err!(BlogError::FromNoneErr)
        // };
        // let pre_blog = match *pre_option {
        //     [0] => Option::None,
        //     [1] => Option::Some(Pubkey::new_from_array(*pre_blog)),
        //     _ => return err!(BlogError::FromPreBlogErr)
        // };

        // let encrypted = match *encrypted {
        //     [0] => false,
        //     [1] => true,
        //     _ => return err!(BlogError::FromEncryptedErr)
        // };

        // let created_time = i64::from_le_bytes(*created_time);
        // let title = match String::from_utf8(title.to_vec()) {
        //     Ok(k) => k,
        //     Err(_) => return err!(BlogError::FromTitleErr)
        // };
        // let tag = match *pre_tag {
        //     [0] => Option::None,
        //     [1] => Option::Some(Pubkey::new_from_array(*tag)),
        //     _ => return err!(BlogError::FromTagErr)
        // };
        // let uuid = match *pre_uuid {
        //     [0] => Option::None,
        //     [1] => String::from_utf8(uuid.to_vec()).ok(),
        //     _ => return err!(BlogError::FromTagErr)
        // };

        // msg!("owner: {:?}", owner.to_string());
        // msg!("key: {:?}", key);
        // msg!("nonce: {:?}", nonce);
        // msg!("encrypted: {:?}", encrypted);
        // if let Some(pre_blog) = pre_blog {
        //     msg!("pre_blog: {:?}", pre_blog.to_string());
        // } else {
        //     msg!("pre_blog: None");
        // }
        // msg!("created_time: {:?}", created_time.to_string());
        // msg!("title: {:?}", title);
        // if let Some(tag) = tag {
        //     msg!("tag: {:?}", tag.to_string());
        // } else {
        //     msg!("tag: None");
        // }
        // if let Some(uuid) = uuid.clone() {
        //     msg!("uuid: {:?}", uuid);
        // } else {
        //     msg!("uuid: None");
        // }
        // Ok(Blog {
        //     owner, key, nonce, encrypted, pre_blog, created_time, title, tag, uuid,
        // })
    }
}

// pub fn meta_deser(buf: &mut &[u8]) -> Result<Blog> {
// }

// impl Sealed for Blog {}
// impl IsInitialized for Blog {
//     fn is_initialized(&self) -> bool {
//         !self.key.is_empty()
//     }
// }
// impl Pack for Blog {
//     const LEN: usize = 120;//Self::space();
//     fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
//         let src = array_ref![src, 0, 120];
//         let (_anchor_pre, owner, pre, _string_pre,key) =
//             array_refs![src, 8, 32, 33, 4, 43];
//         let pre_blog = unpack_option_key(pre)?;
//         // let supply = u64::from_le_bytes(*supply);
//         let key = match  String::from_utf8(key.to_vec()) {
//             Ok(k) => k,
//             Err(_) => return err!(BlogError::CollectionNotVerified)
//         };
//         Ok(Blog {
//             owner: Pubkey::new_from_array(*owner),
//             pre: pre_blog,
//             key,
//         })
//     }
    // fn pack_into_slice(&self, dst: &mut [u8]) {
    //     let dst = array_mut_ref![dst, 0, 82];
    //     let (
    //         mint_authority_dst,
    //         supply_dst,
    //         decimals_dst,
    //         is_initialized_dst,
    //         freeze_authority_dst,
    //     ) = mut_array_refs![dst, 36, 8, 1, 1, 36];
    //     let &Mint {
    //         ref mint_authority,
    //         supply,
    //         decimals,
    //         is_initialized,
    //         ref freeze_authority,
    //     } = self;
    //     pack_coption_key(mint_authority, mint_authority_dst);
    //     *supply_dst = supply.to_le_bytes();
    //     decimals_dst[0] = decimals;
    //     is_initialized_dst[0] = is_initialized as u8;
    //     pack_coption_key(freeze_authority, freeze_authority_dst);
    // }
// }

// fn unpack_option_key(src: &[u8; 33]) -> Result<COption<Pubkey>, ProgramError> {
//     let (tag, body) = array_refs![src, 1, 32];
//     match *tag {
//         [0] => Ok(Option::None),
//         [1] => Ok(Option::Some(Pubkey::new_from_array(*body))),
//         _ => Err(ProgramError::InvalidAccountData),
//     }
// }


#[derive(Accounts)]
#[instruction(nonce: String)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = payer,
        space = Blog::space(),
        seeds = [b"d-blog", nonce.as_bytes(), payer.key().as_ref()],
        bump
    )]
    pub blog: Account<'info, Blog>,

    /// CHECK: is not written to or read
    pub owner: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
