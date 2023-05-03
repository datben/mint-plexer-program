use anchor_lang::{prelude::*};
use anchor_spl::{token::{Mint, Token, TokenAccount}, associated_token::AssociatedToken};

mod err;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod mint_plexer_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,_main_mint_decimals: u8) -> Result<()> {
        let multi_plexer = &mut ctx.accounts.mint_plexer;
        multi_plexer.authority = ctx.accounts.authority.key();
        multi_plexer.main_mint = ctx.accounts.main_mint.key();
        Ok(())
    }

    pub fn add_pair(ctx: Context<AddPair>)-> Result<()> {
        let multi_plexer = &mut ctx.accounts.mint_plexer;
        multi_plexer.twin_mint_pairs.push(ctx.accounts.new_pair.key());
        Ok(())
    }

    pub fn convert_to_main(ctx:Context<Convert>)-> Result<()> {
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct MintPlexer {
    pub main_mint: Pubkey,

    pub authority: Pubkey,

    pub twin_mint_pairs: Vec<Pubkey>,
}

impl MintPlexer {
    pub const LEN: usize = 10000;

    pub fn is_twin_pair(&self,mint:&Pubkey)->bool{
        self.twin_mint_pairs.contains(mint)
    }
}

#[derive(Accounts)]
#[instruction(_main_mint_decimals: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,    
        payer = authority,
        mint::decimals = _main_mint_decimals,
        mint::authority = mint_plexer
    )]
    pub main_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        space = MintPlexer::LEN,
        seeds = [
            b"mint_plexer".as_ref(),
            main_mint.key().as_ref()
        ],
        bump,
        payer = authority
    )]
    pub mint_plexer: Box<Account<'info, MintPlexer>>,


    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct AddPair<'info> {
    #[account(mut, address = mint_plexer.authority)]
    pub authority: Signer<'info>,

    #[account(address = mint_plexer.main_mint)]
    pub main_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            b"mint_plexer".as_ref(),
            main_mint.key().as_ref()
        ],
        bump,
    )]
    pub mint_plexer: Box<Account<'info, MintPlexer>>,

    #[account(constraint = new_pair.decimals == main_mint.decimals)]
    pub new_pair: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = main_mint,
        associated_token::authority = authority,
    )]
    pub new_pair_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct Convert<'info> {
    #[account(mut, address = mint_plexer.main_mint)]
    pub main_mint: Box<Account<'info, Mint>>,

    #[account(constraint =  mint_plexer.is_twin_pair(&twin_pair.key()))]
    pub twin_pair: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            b"mint_plexer".as_ref(),
            main_mint.key().as_ref()
        ],
        bump,
    )]
    pub mint_plexer: Box<Account<'info, MintPlexer>>,
}