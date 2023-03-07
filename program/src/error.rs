use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Copy, Clone, Debug)]
pub enum LoanError {
    #[error("Invalid Data")]
    InvalidData,
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Wrong Loan Amount Provided")]
    WrongLoanAmount,
    #[error("Loan Request Is Already Compeleted")]
    LoanRequestAlreadyCompeleted,
    #[error("Not Enough Balance To Provide Loan")]
    NotEnoughBalanceToProvideLoan,
    #[error("Loan Request has already Initiated")]
    RequestAlreadyInitialized,
    #[error("Loan has not been granted")]
    LoanHasNotGranted,
    #[error("Loan Transaction has been compeleted")]
    LoanTransactionHasAlreadyCompeleted,
    #[error("Not Enough Balance for Transaction")]
    NotEnoughBalance,
    #[error("Deadline for loan has been passed")]
    LoanDeadlinePassed,
    #[error("Loan has either granted or request has not been initialized")]
    WrongStage,
    #[error("Cannot claim nft untill deadline is passed")]
    NoClaim,
}

impl From<LoanError> for ProgramError {
    fn from(e: LoanError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
