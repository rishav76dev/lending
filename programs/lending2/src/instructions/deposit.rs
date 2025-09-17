use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };
use crate::state::{ User, Bank};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, //signer of instruction for Deposit instruction

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [b"treasury", mint.key().as_ref()],
        bump,
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>, //on deposit token into a bank , that token us sent to bank token account

    #[account(
        mut,
        seeds = [signer.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, User>, // used for storing informatin of user using the lending protocol

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,  //will be to transfer token from user to bank token account

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


pub fn process_deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let transfer_cpi_accounts = TransferChecked{
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(), //singer of intrustion because own the user token account
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);

    let decimals = ctx.accounts.mint.decimals;
    token_interface::transfer_checked(
        cpi_ctx,
        amount,
        decimals,
    )?;

    //transfer has taken place, now we need to  update the bank and user accounts

    let bank = &mut ctx.accounts.bank;

    if bank.total_deposits == 0 {
        bank.total_deposits = amount;
        bank.total_deposited_shares = amount;
    }
    let deposit_ratio = amount.checked_div(bank.total_deposits).unwrap();
    let user_shares = bank.total_deposited_shares.checked_mul(deposit_ratio).unwrap();

    let user = &mut ctx.accounts.user_account;

    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            user.deposited_usdc += amount;
            user.deposited_usdc_shares += user_shares;
        },
        _=> {
            user.deposited_sol += amount;
            user.deposited_sol_shares += user_shares;
        }

    }

        bank.total_deposits += amount;
        bank.total_deposited_shares += user_shares;
    // note : lending protocol -> 2 options when we deposit
    // 1) they eithe mint a collateral token to represent collateral that user has in the bank
    // 2) they create a share for us to represent portion of bank user know
    // for this project we are creating share instead ,because we will have to burn and mint token throught out the protocol
    user.last_update = Clock::get()?.unix_timestamp;
    Ok(())
}
