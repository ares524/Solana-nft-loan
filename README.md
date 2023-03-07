# SoLoana NFT - P2P

# 🔖 About

Soloana NFT allows user to put their NFT's as collateral and take a loan , if the borrower able to payback the loan before the deadline he specified in the request then borrower will get his NFT back else the NFT can be claimed by the Lender

### How it Works

Soloana nft as it sounds relies on solana , it takes the account holding the nft's and give it's authority to a vault PDA and creates a request for loan with a defined deadline by borrower, lender then can see the nft and deadline , and provides the loan to the borrower , now the time starts and if the deadline passes borrower can no longer clain the nft by paying loan back and lender can anytime claim the nft

## 🚀 Features

- Borrow stable coins using nft as collateral
- Pay loan and get payback with interest
- Claim nft if deadline pass

## 🔥 Getting Started

### Prerequisites

- <a href="https://docs.solana.com/cli/install-solana-cli-tools">Solana</a>

### Installation

- Fork the Repository

```
   $ git clone https://github.com/atharmohammad/Solana-File-Sharing-System.git
   $ cd Sol-Loan-a-NFT
   $ git remote add upstream https://github.com/atharmohammad/Solana-File-Sharing-System.git
   $ npm install
   $ npm run build:program
   $ npm run deploy:program
   $ npm run start
```
