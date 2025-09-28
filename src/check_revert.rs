use alloy_network::ReceiptResponse;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ReceiptExtError<R>
where
    R: std::fmt::Debug,
{
    #[error("Transaction reverted: {:?}", _0)]
    TransactionReverted(R),
}
pub trait ReceiptExt: ReceiptResponse + Sized {
    fn check_revert(self) -> Result<Self, ReceiptExtError<Self>>
    where
        Self: Sized + std::fmt::Debug,
    {
        if self.status() {
            Ok(self)
        } else {
            Err(ReceiptExtError::TransactionReverted(self))
        }
    }
}

impl<R: ReceiptResponse> ReceiptExt for R {}
