use crate::id;
use crate::state::*;
use crate::{error::LoanError, instructions::*};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::invoke_signed;
use solana_program::system_instruction::create_account;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_token::instruction::{set_authority, transfer};

fn check_loan_stage(stage: Stage) -> Result<Stage, ProgramError> {
    match stage {
        Stage::INITIALIZED => return Err(LoanError::LoanHasNotGranted.into()),
        Stage::LOANPAIDBACK => return Err(LoanError::LoanRequestAlreadyCompeleted.into()),
        Stage::DEADLINEPASSED => return Err(LoanError::LoanDeadlinePassed.into()),
        _ => Ok(Stage::LOANGRANTED),
    }
}

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    msg!("program starts!");
    let instruction = Payload::try_from_slice(input)?;
    let state_len = 1 + 32 + 32 + 32 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 8;
    match instruction.variant {
        0 => {
            msg!("Initialize request instruction starts !");
            let accounts_iter = &mut accounts.iter();
            let borrower = next_account_info(accounts_iter)?;
            let borrower_token_account = next_account_info(accounts_iter)?;
            let principal_token = next_account_info(accounts_iter)?;
            let nft_holding_token_account = next_account_info(accounts_iter)?;
            let collateral_nft = next_account_info(accounts_iter)?;
            let vault = next_account_info(accounts_iter)?;
            let loan_request_state = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;
            let system_program_acc = next_account_info(accounts_iter)?;
            let rent = &Rent::from_account_info(next_account_info(accounts_iter)?)?;
            msg!("Create state account !");
            let str = collateral_nft.key.clone();
            let request_state_seeds = vec![&b"state"[..], str.as_ref()];
            let (_state_pda, _state_bump) =
                Pubkey::find_program_address(request_state_seeds.as_slice(), &id());
            let create_state_acc_inst = create_account(
                borrower.key,
                loan_request_state.key,
                rent.minimum_balance(state_len),
                state_len as u64,
                &id(),
            );
            invoke_signed(
                &create_state_acc_inst,
                &[
                    borrower.clone(),
                    loan_request_state.clone(),
                    system_program_acc.clone(),
                ],
                &[&[&b"state"[..], str.as_ref(), &[_state_bump]]],
            )?;
            if !rent.is_exempt(loan_request_state.lamports(), loan_request_state.data_len()) {
                return Err(ProgramError::AccountNotRentExempt);
            }
            msg!("Deserialize the loan request state account!");
            let mut request_info = Request::unpack_unchecked(*loan_request_state.data.borrow())?;
            if request_info.stage != Stage::UNINITIALIZED {
                return Err(LoanError::RequestAlreadyInitialized.into());
            }
            request_info.borrower = *borrower.key;
            request_info.borrower_token_account = *borrower_token_account.key;
            request_info.collateral_nft = *collateral_nft.key;
            request_info.nft_holding_account = *nft_holding_token_account.key;
            request_info.stage = Stage::INITIALIZED;
            request_info.vault = *vault.key;
            request_info.loan_amount = instruction.arg1;
            request_info.deadline = instruction.arg2;
            request_info.principal_token = *principal_token.key;
            request_info.lender_token_account = id(); // placeholder for lender's token account
            request_info.lender = id(); // placeholder for the lender
            request_info.loan_submission_time = 0; // placeholder , will be set when someone grants the loan
            msg!("{},{}", instruction.arg1, instruction.arg2);
            msg!("Serialize the loan request state account after assigning the values");
            request_info.serialize(&mut &mut loan_request_state.data.borrow_mut()[..])?;
            let state_seeds = vec![b"vault".as_ref(), collateral_nft.key.as_ref()];
            let (vault_pda, _bump) = Pubkey::find_program_address(state_seeds.as_slice(), &id());
            msg!("transfer authority of token account holding collateral nft to vault pda!"); // create vault pda for every request by considering nft mint as the seed for the pda
            let transfer_nft_authority = set_authority(
                &token_program.key,
                &nft_holding_token_account.key,
                Some(&vault_pda),
                spl_token::instruction::AuthorityType::AccountOwner,
                &borrower.key,
                &[&borrower.key],
            )?;
            invoke(
                &transfer_nft_authority,
                &[
                    token_program.clone(),
                    nft_holding_token_account.clone(),
                    borrower.clone(),
                ],
            )?;
            Ok(())
        }
        1 => {
            msg!("Compelete the request by providing loan instruction starts !");
            let accounts_iter = &mut accounts.iter();
            let lender = next_account_info(accounts_iter)?;
            let borrower_token_account = next_account_info(accounts_iter)?;
            let lenders_token_account = next_account_info(accounts_iter)?;
            let loan_request_state = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;
            let clock = Clock::from_account_info(next_account_info(accounts_iter)?)?;
            if *lenders_token_account.owner != spl_token::id() {
                return Err(ProgramError::IllegalOwner);
            }
            let lenders_token_state = spl_token::state::Account::unpack_unchecked(
                *lenders_token_account.try_borrow_data()?,
            )?;
            msg!("Deserialize the loan request state account!");
            let mut request_info = Request::unpack_unchecked(*loan_request_state.data.borrow())?;
            msg!("Check if the lender have enough tokens to provide loan !");
            if lenders_token_state.amount < request_info.loan_amount {
                return Err(LoanError::NotEnoughBalanceToProvideLoan.into());
            }

            msg!("check if the amount the lender is lending is correct !");
            if instruction.arg1 != request_info.loan_amount {
                return Err(LoanError::WrongLoanAmount.into());
            }
            msg!("check if no one has provided loan before !");
            if request_info.stage != Stage::INITIALIZED {
                return Err(LoanError::LoanRequestAlreadyCompeleted.into());
            }
            request_info.lender = *lender.key;
            request_info.lender_token_account = *lenders_token_account.key;
            request_info.stage = Stage::LOANGRANTED;
            request_info.loan_submission_time =
                clock.unix_timestamp as u64 + request_info.deadline * 24 * 60 * 60; // adding days needed to payback loan
            msg!(
                "loan submission time is : {}",
                request_info.loan_submission_time
            );
            msg!("{}", instruction.arg1);
            msg!("Serialize the loan request state account after assigning the values");
            request_info.serialize(&mut &mut loan_request_state.data.borrow_mut()[..])?;
            msg!("transfer the loan amount to the borrower !");
            let transfer_loan = transfer(
                token_program.key,
                lenders_token_account.key,
                &request_info.borrower_token_account,
                lender.key,
                &[&lender.key],
                request_info.loan_amount,
            )?;
            invoke(
                &transfer_loan,
                &[
                    token_program.clone(),
                    lenders_token_account.clone(),
                    lender.clone(),
                    borrower_token_account.clone(),
                ],
            )?;
            Ok(())
        }
        2 => {
            msg!("Payback the loan amount instruction start !");
            let accounts_iter = &mut accounts.iter();
            let borrower = next_account_info(accounts_iter)?;
            let borrower_token_account = next_account_info(accounts_iter)?;
            let lenders_token_account = next_account_info(accounts_iter)?;
            let vault = next_account_info(accounts_iter)?;
            let loan_request_state = next_account_info(accounts_iter)?;
            let nft_holding_token_account = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;
            let clock = Clock::from_account_info(next_account_info(accounts_iter)?)?;
            msg!("Deserialize the loan request state account!");
            let request_info = Request::unpack_unchecked(*loan_request_state.data.borrow())?;
            let loan_stage = check_loan_stage(request_info.stage);
            if loan_stage.is_err() {
                return Err(loan_stage.unwrap_err());
            }
            let borrowers_token_account_data = spl_token::state::Account::unpack_unchecked(
                *borrower_token_account.try_borrow_data()?,
            )?;
            if borrowers_token_account_data.amount < request_info.loan_amount {
                return Err(LoanError::NotEnoughBalance.into());
            }
            if clock.unix_timestamp as u64 - 30 > request_info.loan_submission_time {
                // giving some seconds more just to make sure that deadline is not elapsed during transaction processing
                return Err(LoanError::LoanDeadlinePassed.into());
            }
            msg!("transfer nft back to borrower !");
            let state_seeds = vec![b"vault".as_ref(), request_info.collateral_nft.as_ref()];
            let (_vault_pda, _bump) = Pubkey::find_program_address(state_seeds.as_slice(), &id());
            let tranfer_nft = set_authority(
                &token_program.key,
                &request_info.nft_holding_account,
                Some(&borrower.key),
                spl_token::instruction::AuthorityType::AccountOwner,
                &request_info.vault,
                &[&request_info.vault],
            )?;
            invoke_signed(
                &tranfer_nft,
                &[
                    token_program.clone(),
                    nft_holding_token_account.clone(),
                    borrower.clone(),
                    vault.clone(),
                ],
                &[&[
                    &b"vault"[..],
                    request_info.collateral_nft.as_ref(),
                    &[_bump],
                ]],
            )?;
            msg!("transfer the loan amount back to the lender !");
            let transfer_loan = transfer(
                &token_program.key,
                &borrower_token_account.key,
                &request_info.lender_token_account,
                &request_info.borrower,
                &[&request_info.borrower],
                request_info.loan_amount,
            )?;
            invoke(
                &transfer_loan,
                &[
                    token_program.clone(),
                    borrower_token_account.clone(),
                    borrower.clone(),
                    lenders_token_account.clone(),
                ],
            )?;
            msg!("close the accounts and release the rent !");
            // since loan state account is owned by the program only so we can just deduct and credit all lamports back to borrower !
            let dest_starting_lamports = borrower.lamports();
            **borrower.lamports.borrow_mut() = dest_starting_lamports
                .checked_add(loan_request_state.lamports())
                .unwrap();
            **loan_request_state.lamports.borrow_mut() = 0;

            let mut source_data = loan_request_state.data.borrow_mut();
            source_data.fill(0);


            Ok(())
        },
        3 => {
            msg!("Cancel the request instruction start !");
            let accounts_iter = &mut accounts.iter();
            let borrower = next_account_info(accounts_iter)?;
            let vault = next_account_info(accounts_iter)?;
            let nft_holding_token_account = next_account_info(accounts_iter)?;
            let loan_request_state = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;

            msg!("Deserialize the loan request state account!");
            let request_info = Request::unpack_unchecked(*loan_request_state.data.borrow())?;
            if request_info.stage != Stage::INITIALIZED {
                return Err(LoanError::WrongStage.into());
            }
            msg!("transfer nft back to borrower !");
            let state_seeds = vec![b"vault".as_ref(), request_info.collateral_nft.as_ref()];
            let (_vault_pda, _bump) = Pubkey::find_program_address(state_seeds.as_slice(), &id());
            let tranfer_nft = set_authority(
                &token_program.key,
                &request_info.nft_holding_account,
                Some(&borrower.key),
                spl_token::instruction::AuthorityType::AccountOwner,
                &request_info.vault,
                &[&request_info.vault],
            )?;
            invoke_signed(
                &tranfer_nft,
                &[
                    token_program.clone(),
                    nft_holding_token_account.clone(),
                    borrower.clone(),
                    vault.clone(),
                ],
                &[&[
                    &b"vault"[..],
                    request_info.collateral_nft.as_ref(),
                    &[_bump],
                ]],
            )?;
            msg!("close the accounts and release the rent !");
            // since loan state account is owned by the program only so we can just deduct and credit all lamports back to borrower !
            let dest_starting_lamports = borrower.lamports();
            **borrower.lamports.borrow_mut() = dest_starting_lamports
                .checked_add(loan_request_state.lamports())
                .unwrap();
            **loan_request_state.lamports.borrow_mut() = 0;

            let mut source_data = loan_request_state.data.borrow_mut();
            source_data.fill(0);

            Ok(())
        },
        4 => {
            msg!("Claim the Collateral Instruction starts !");
            let accounts_iter = &mut accounts.iter();
            let lender = next_account_info(accounts_iter)?;
            let vault = next_account_info(accounts_iter)?;
            let nft_holding_token_account = next_account_info(accounts_iter)?;
            let loan_request_state = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;
            let clock = Clock::from_account_info(next_account_info(accounts_iter)?)?;

            msg!("Deserialize the loan request state account!");
            let mut request_info = Request::unpack_unchecked(*loan_request_state.data.borrow())?;
            if request_info.stage != Stage::LOANGRANTED {
                return Err(LoanError::WrongStage.into());
            }
            if clock.unix_timestamp as u64 - 30 <= request_info.loan_submission_time {
                // giving some seconds more just to make sure that deadline is not elapsed during transaction processing
                return Err(LoanError::NoClaim.into());
            }
            if *lender.key != request_info.lender {
                return Err(ProgramError::IllegalOwner);
            }
            msg!("Transfer nft to lender !");
            let state_seeds = vec![b"vault".as_ref(), request_info.collateral_nft.as_ref()];
            let (_vault_pda, _bump) = Pubkey::find_program_address(state_seeds.as_slice(), &id());
            let tranfer_nft = set_authority(
                &token_program.key,
                &request_info.nft_holding_account,
                Some(&lender.key),
                spl_token::instruction::AuthorityType::AccountOwner,
                &request_info.vault,
                &[&request_info.vault],
            )?;
            invoke_signed(
                &tranfer_nft,
                &[
                    token_program.clone(),
                    nft_holding_token_account.clone(),
                    lender.clone(),
                    vault.clone(),
                ],
                &[&[
                    &b"vault"[..],
                    request_info.collateral_nft.as_ref(),
                    &[_bump],
                ]],
            )?;
            request_info.stage = Stage::DEADLINEPASSED;
            msg!("Serialize the loan request state account after assigning the values");
            request_info.serialize(&mut &mut loan_request_state.data.borrow_mut()[..])?;
            Ok(())
        }
        _ => return Err(ProgramError::InvalidArgument),
    }
}
