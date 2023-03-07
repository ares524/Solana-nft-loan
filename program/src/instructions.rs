use borsh::{BorshDeserialize, BorshSerialize};
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
pub struct Payload {
    pub variant: u8,
    pub arg1: u64,
    pub arg2: u64,
}
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, PartialEq)]
pub enum LoanInstruction {
    /// Initialize request for a loan and provides with amount of loan required and deadline which can be followed
    /// accounts required :
    /// 0 - [signer] borrower who is initiating request for a loan
    /// 1 - [] borrower token account which will receive the loan
    /// 2 - [] principal token which borrower needs
    /// 3 - [writer] token account which holds the nft
    /// 4 - [writer] mint of the collateral nft
    /// 5 - [writer] vault which will store the nft
    /// 6 - [writer] loan request state for the nft
    /// 7 - [] token program
    /// 8 - [] rent sysvar
    InitializeRequest { loan_amount: u64, deadline: u64 },
    /// Accept a request and provide the loans
    /// accounts required :
    /// 0 - [signer] lender who is providing the loan
    /// 1 - [writer] borrower token account which will receive the loan
    /// 2 - [writer] token account which will provide loan amount
    /// 3 - [writer] loan request state for the nft
    /// 4 - [] token program
    /// 5 - [] clock sysvar
    CompeleteRequest { amount: u64 },
    /// Pay the amount of token taken as loan back and get back the collateral nft
    /// accounts required :
    /// 0 - [signer] borrower who had taken the loan
    /// 1 - [writer] borrower token account which will payback the principal token amount
    /// 2 - [writer] lenders receiving token account which will receive the amount
    /// 3 - [writer] vault which stored the nft
    /// 4 - [writer] loan request state for the nft
    /// 5 - [writer] nft holding token account
    /// 6 - [] token program
    /// 7 - [] clock sysvar
    PaybackLoan,
    /// Cancel the request for loan if not granted
    /// accounts required :
    /// 0 - [signer] borrower who initiated the request for loan
    /// 1 - [writer] vault pda that is authority for the nft token account
    /// 2 - [writer] nft token account that is holiding the collateral nft
    /// 3 - [writer] loan request state for the nft
    /// 4 - [] token program
    CancelRequest,
    /// Claim the nft if the deadline has been exceeded
    /// accounts required :
    /// 0 - [signer] lender who provided the loan
    /// 1 - [writer] vault pda that is authority for the nft token account
    /// 2 - [writer] nft token account that is holiding the collateral nft
    /// 3 - [writer] loan request state for the nft
    /// 4 - [] token program
    /// 5 - [] clock sysvar
    ClaimCollateral,
}

// pub fn initialize(
//     user_sender: &Pubkey,
//     senders_token_account: &Pubkey,
//     escrow_token_account: &Pubkey,
//     escrow_account: &Pubkey,
//     rent: &Pubkey,
//     token_program: &Pubkey,
//     amount: u64,
// ) -> Instruction {
//     Instruction::new_with_borsh(
//         id(),
//         &EscrowInstruction::Initialize { amount },
//         vec![
//             AccountMeta::new(*user_sender, true),
//             AccountMeta::new(*senders_token_account, false),
//             AccountMeta::new(*escrow_token_account, false),
//             AccountMeta::new(*escrow_account, false),
//             AccountMeta::new(*rent, false),
//             AccountMeta::new(*token_program, false),
//         ],
//     )
// }
