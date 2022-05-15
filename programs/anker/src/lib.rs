use anchor_lang::prelude::*;
use anchor_spl::{self, associated_token::{AssociatedToken}, token::{self, Mint, TokenAccount, Token, transfer, Transfer}};

declare_id!("AM8FqXXJeknhcoHk2im2vqzbkveLVq1GSr2kvXsoKRkR");

const PREFIX: &str = "rogue_swapper";

#[program]
pub mod anker {
    use super::*;

    pub fn init_market<'info>(
        ctx: Context<'_, '_, '_, 'info, InitMarket<'info>>,
        bump: u8,
        item_quantity: u64,
        per_item_price: u64
        ) -> Result<()> {
        
        let creator = &mut ctx.accounts.creator;

        let item = &ctx.accounts.item;
        let token = &ctx.accounts.token;

        let creator_token_associated_account = &ctx.accounts.creator_token_associated_account;
        let creator_item_associated_account = &mut ctx.accounts.creator_item_associated_account;
        let item_associated_account = &mut ctx.accounts.item_associated_account;
        let token_program = &ctx.accounts.token_program;
        let market_acc = &mut ctx.accounts.market;

        transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: creator_item_associated_account.to_account_info(),
                    to: item_associated_account.to_account_info(),
                    authority: creator.to_account_info(),
                },
            ),
            item_quantity,
        )?;

        market_acc.active = false;
        market_acc.price = per_item_price;

        Ok(())
    }

    pub fn edit_market<'info>(
        ctx: Context<'_, '_, '_, 'info, EditMarket<'info>>,
        bump: u8,
        active: Option<bool>,
        per_item_price: Option<u64>,
        ) -> Result<()> {

        let market_acc = &mut ctx.accounts.market;
        if let Some(x) = active {
            market_acc.active = x;
        }
        if let Some(x) = per_item_price {
            market_acc.price = x;
        }
        Ok(())
    }

    pub fn buy_item<'info>(
        ctx: Context<'_, '_, '_, 'info, BuyItem<'info>>,
        bump: u8,
        item_quantity: u64,
        ) -> Result<()> {

        let market_acc = &mut ctx.accounts.market;
        if market_acc.active == false {
            return Err(error!(ErrorCode::MarketNotActive))    
        }

        let total_spl_token_price = market_acc.price * item_quantity;
        let buyer = &mut ctx.accounts.buyer;
        let creator = &ctx.accounts.creator;
        let item = &ctx.accounts.item;
        let token = &ctx.accounts.token;
        let token_program = &ctx.accounts.token_program;

        let item_associated_account = &mut ctx.accounts.item_associated_account;
        let creator_token_associated_account = &mut ctx.accounts.creator_token_associated_account;
        let buyer_item_associated_account = &mut ctx.accounts.buyer_item_associated_account;
        let buyer_token_associated_account = &mut ctx.accounts.buyer_token_associated_account;

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    from: item_associated_account.to_account_info(),
                    to: buyer_item_associated_account.to_account_info(),
                    authority: market_acc.to_account_info(),
                },
                &[&[PREFIX.as_bytes(),
                creator.key().as_ref(),
                item.key().as_ref(),
                token.key().as_ref(),
                bytemuck::bytes_of(&bump)][..]],
            ),
            item_quantity,
        )?;

        transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: buyer_token_associated_account.to_account_info(),
                    to: creator_token_associated_account.to_account_info(),
                    authority: buyer.to_account_info(),
                },
            ),
            total_spl_token_price,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mint::decimals = 0)]
    pub item: Box<Account<'info, Mint>>,
    #[account(mint::decimals = 0)]
    pub token: Box<Account<'info, Mint>>,
    #[account(
        init, payer=creator, space=8+1+8,
        seeds= [
                PREFIX.as_bytes(),
                creator.key().as_ref(),
                item.key().as_ref(),
                token.key().as_ref()
               ],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(init_if_needed, payer=creator, associated_token::mint = item, associated_token::authority=market)]
    pub item_associated_account: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer=creator, associated_token::mint = token, associated_token::authority=creator)]
    pub creator_token_associated_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub creator_item_associated_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EditMarket<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mint::decimals = 0)]
    pub item: Box<Account<'info, Mint>>,
    #[account(mint::decimals = 0)]
    pub token: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds= [
                PREFIX.as_bytes(),
                creator.key().as_ref(),
                item.key().as_ref(),
                token.key().as_ref()
               ],
        bump
    )]
    pub market: Account<'info, Market>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct BuyItem<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: just the creator public key 
    pub creator: UncheckedAccount<'info>,
    #[account(mint::decimals = 0)]
    pub item: Box<Account<'info, Mint>>,
    #[account(mint::decimals = 0)]
    pub token: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds= [
                PREFIX.as_bytes(),
                creator.key().as_ref(),
                item.key().as_ref(),
                token.key().as_ref()
               ],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(mut, associated_token::mint = item, associated_token::authority=market)]
    pub item_associated_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, associated_token::mint = token, associated_token::authority=creator)]
    pub creator_token_associated_account: Box<Account<'info, TokenAccount>>,

    #[account(init_if_needed, payer= buyer, associated_token::mint = token, associated_token::authority=buyer)]
    pub buyer_token_associated_account: Box<Account<'info, TokenAccount>>,

    #[account(init_if_needed, payer= buyer, associated_token::mint = item, associated_token::authority=buyer)]
    pub buyer_item_associated_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}


#[account]
#[derive(Default, Debug)]
pub struct Market {
    pub active: bool,
    pub price: u64
}

#[error_code]
pub enum ErrorCode {
    #[msg("Market isn't active")]
    MarketNotActive,
    #[msg("Market is empty")]
    MarketDepleted,
}
