use anchor_lang::{prelude::*, solana_program::bpf_loader_upgradeable};
use anchor_spl::token::{Mint, Token};
use err::MintPlexerError;
use program::MintPlexerProgram;

mod err;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod mint_plexer_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,_main_mint_decimals: u8) -> Result<()> {
        let multi_plexer = &mut ctx.accounts.mint_plexer;
        multi_plexer.autority = ctx.accounts.owner.key();
        multi_plexer.main_mint = ctx.accounts.main_mint.key();
        Ok(())
    }

    pub fn add_pair(ctx: Context<AddPair>)-> Result<()> {

        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct MintPlexer {
    pub main_mint: Pubkey,

    pub autority: Pubkey,

    pub mint_pairs: Vec<Pubkey>,
}

impl MintPlexer {
    pub const LEN: usize = 10000;
}

#[derive(Accounts)]
#[instruction(_main_mint_decimals: u8)]
pub struct Initialize<'info> {
    #[account(mut, address = mint_plexer_program_data.upgrade_authority_address.ok_or(MintPlexerError::AccessDenied)?)]
    pub owner: Signer<'info>,

    #[account(
        init,    
        payer = owner,
        mint::decimals = _main_mint_decimals,
        mint::authority = mint_plexer
    )]
    pub main_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        space = MintPlexer::LEN,
        seeds = [
            b"config".as_ref(),
        ],
        bump,
        payer = owner
    )]
    pub mint_plexer: Box<Account<'info, MintPlexer>>,

    #[account(
        seeds = [MintPlexerProgram::id().as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID 
    )]
    pub mint_plexer_program_data: Box<Account<'info, ProgramData>>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct AddPair<'info> {
    #[account(address = mint_plexer.main_mint)]
    pub main_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            b"config".as_ref(),
        ],
        bump,
    )]
    pub mint_plexer: Box<Account<'info, MintPlexer>>,

    pub new_pair: Box<Account<'info, Mint>>,
}