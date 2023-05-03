use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};

mod err;

declare_id!("DcoHFXZHaLRQ2B37Bqc7afMpPr8T9VULyNdGj87wctcv");

#[program]
pub mod mint_plexer_program {
    use anchor_spl::token::{Burn, MintTo, Transfer};
  
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump:u8, _main_mint_decimals: u8) -> Result<()> {
        let multi_plexer = &mut ctx.accounts.mint_plexer;
        multi_plexer.authority = ctx.accounts.authority.key();
        multi_plexer.main_mint = ctx.accounts.main_mint.key();
        multi_plexer.bump = [bump];
        Ok(())
    }

    pub fn add_pair(ctx: Context<AddPair>) -> Result<()> {
        let multi_plexer = &mut ctx.accounts.mint_plexer;
        multi_plexer
            .twin_mint_pairs
            .push(ctx.accounts.new_pair.key());
        Ok(())
    }

    pub fn convert_to_main(ctx: Context<Convert>, amount: u64) -> Result<()> {
        // transfer user -> program
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    to: ctx
                        .accounts
                        .program_twin_pair_token_account
                        .to_account_info(),
                    from: ctx.accounts.user_twin_pair_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // program mints amount token to user
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.main_mint.to_account_info(),
                    to: ctx.accounts.user_main_mint_token_account.to_account_info(),
                    authority: ctx.accounts.mint_plexer.to_account_info(),
                },
                &[&ctx.accounts.mint_plexer.seed()],
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn convert_from_main(ctx: Context<Convert>, amount: u64) -> Result<()> {
        if ctx.accounts.program_twin_pair_token_account.amount < amount {
            return err!(err::MintPlexerError::NotEnoughLiquidity);
        }

        // user burns amount token
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    from: ctx.accounts.user_main_mint_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.main_mint.to_account_info(),
                },
            ),
            amount,
        )?;

        // transfer program -> user
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    to: ctx
                        .accounts
                        .program_twin_pair_token_account
                        .to_account_info(),
                    from: ctx.accounts.user_twin_pair_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct MintPlexer {
    pub bump: [u8;1],

    pub main_mint: Pubkey,

    pub authority: Pubkey,

    pub twin_mint_pairs: Vec<Pubkey>,
}

impl MintPlexer {
    pub const LEN: usize = 10000;

    pub fn is_twin_pair(&self, mint: &Pubkey) -> bool {
        self.twin_mint_pairs.contains(mint)
    }

    pub fn seed(&self) -> [&[u8]; 3] {
        [b"mint_plexer".as_ref(), self.main_mint.as_ref(), self.bump.as_ref()]
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
    #[account(
        mut,
        seeds = [
            b"mint_plexer".as_ref(),
            main_mint.key().as_ref()
        ],
        bump,
    )]
    pub mint_plexer: Box<Account<'info, MintPlexer>>,

    #[account(mut, address = mint_plexer.authority)]
    pub authority: Signer<'info>,

    #[account(address = mint_plexer.main_mint)]
    pub main_mint: Box<Account<'info, Mint>>,

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

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Convert<'info> {
    #[account(
        mut,
        seeds = [
            b"mint_plexer".as_ref(),
            main_mint.key().as_ref()
        ],
        bump,
    )]
    pub mint_plexer: Box<Account<'info, MintPlexer>>,

    pub user: Signer<'info>,

    #[account(mut, address = mint_plexer.main_mint)]
    pub main_mint: Box<Account<'info, Mint>>,

    #[account(constraint =  mint_plexer.is_twin_pair(&twin_pair.key()))]
    pub twin_pair: Box<Account<'info, Mint>>,

    #[account(mut, token::mint = main_mint, token::authority = mint_plexer)]
    pub program_twin_pair_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::mint = main_mint, token::authority = user)]
    pub user_main_mint_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::mint = twin_pair, token::authority = user)]
    pub user_twin_pair_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
