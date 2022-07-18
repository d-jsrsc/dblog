use anchor_lang::prelude::*;

#[error_code]
pub enum BlogError {
    #[msg("IncurrentOwner")]
    IncurrentOwner,
    #[msg("PreOwnerIsNotCurrentOwner")]
    PreOwnerIsNotCurrentOwner,
    #[msg("FromKeyErr")]
    FromKeyErr,
    #[msg("FromNoneErr")]
    FromNoneErr,
    #[msg("FromPreBlogErr")]
    FromPreBlogErr,
    #[msg("FromEncryptedErr")]
    FromEncryptedErr,
    #[msg("FromTitleErr")]
    FromTitleErr,
    #[msg("FromTagErr")]
    FromTagErr,
    #[msg("FromUuidErr")]
    FromUuidErr,
    #[msg("TitleMaxLen")]
    TitleMaxLenLimit,
    #[msg("EncryptWordLenLimit")]
    EncryptWordLenLimit,
}
