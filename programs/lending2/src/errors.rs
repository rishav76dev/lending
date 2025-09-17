use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
  #[msg("Insufficient Balance")]
  InsufficientBalance,
  #[msg("Requested amount exceeds borrowable amount")]
  OverBorrowableAmount,
  #[msg("Requested amount exceeds repayable amount")]
  OverRepay
}
