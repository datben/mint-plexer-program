use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq)]
pub enum MintPlexerError {
    #[msg("")]
    AccessDenied,

    #[msg("")]
    NotEnoughLiquidity,
}
