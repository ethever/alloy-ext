use {
    alloy_network::Network,
    alloy_provider::{
        PendingTransactionBuilder, Provider, RootProvider, WalletProvider,
        transport::TransportResult,
    },
    std::marker::PhantomData,
};

#[derive(Debug, Clone)]
pub struct ReadWriteProvider<P1, P2, N> {
    _phantom: PhantomData<N>,
    read: P1,
    write: P2,
}

impl<P1, P2, N> ReadWriteProvider<P1, P2, N>
where
    P1: Provider<N> + Clone,
    P2: Provider<N> + Clone,
    N: Network,
{
    /// Create a new `ReadWriteProvider` with the given read and write clients.
    pub fn new(read: P1, write: P2) -> Self {
        Self {
            read,
            write,
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<P1, P2, N> Provider<N> for ReadWriteProvider<P1, P2, N>
where
    P1: Provider<N>,
    P2: Provider<N>,
    N: Network,
{
    fn root(&self) -> &RootProvider<N> {
        tracing::trace!("Calling read provider.");
        &self.read.root()
    }

    /// All send_transaction calls go to the write endpoint.
    async fn send_transaction(
        &self,
        tx: N::TransactionRequest,
    ) -> TransportResult<PendingTransactionBuilder<N>> {
        tracing::trace!("Sending transaction to write provider.");
        let res = self.write.send_transaction(tx).await?;
        let (_, c) = res.split();

        Ok(PendingTransactionBuilder::from_config(
            self.read.root().clone(),
            c,
        ))
    }

    async fn send_raw_transaction(
        &self,
        encoded_tx: &[u8],
    ) -> TransportResult<PendingTransactionBuilder<N>> {
        tracing::trace!("Sending raw transaction to write provider.");

        let (_, c) = self.write.send_raw_transaction(encoded_tx).await?.split();

        Ok(PendingTransactionBuilder::from_config(
            self.read.root().clone(),
            c,
        ))
    }
}

impl<PRead, PWrite, N> WalletProvider<N> for ReadWriteProvider<PRead, PWrite, N>
where
    PRead: Provider<N>,
    PWrite: WalletProvider<N>,
    N: Network,
{
    type Wallet = PWrite::Wallet;
    fn wallet(&self) -> &Self::Wallet {
        self.write.wallet()
    }

    fn wallet_mut(&mut self) -> &mut Self::Wallet {
        self.write.wallet_mut()
    }
}
