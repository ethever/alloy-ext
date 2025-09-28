use alloy_transport::TransportError;
use alloy_transport_http::{Client, Http};
use eyre::eyre;
use futures::{FutureExt, TryFutureExt, future::select_ok};
use rand::{prelude::IndexedRandom, rng};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use {
    alloy_json_rpc::{RequestPacket, ResponsePacket},
    tower::Service,
};

#[derive(Clone)]
pub struct MultiTransport {
    inner: Vec<Vec<Http<Client>>>,
}

impl MultiTransport {
    pub fn new(inner: Vec<Vec<Http<Client>>>) -> Self {
        Self { inner }
    }
}

impl Service<RequestPacket> for MultiTransport {
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = Pin<Box<dyn Future<Output = Result<ResponsePacket, TransportError>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Always ready
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RequestPacket) -> Self::Future {
        let clients = match self
            .inner
            .iter()
            .cloned()
            .map(|v| v.choose(&mut rng()).cloned())
            .collect::<Option<Vec<_>>>()
        {
            Some(c) => c,
            None => {
                return Box::pin(async {
                    Err(TransportError::LocalUsageError(
                        eyre!("No clients available").into(),
                    ))
                });
            }
        };

        let res = clients.into_iter().map(|mut v| {
            let req_clone = req.clone();
            async move {
                let res = v.call(req_clone.clone()).await;

                match res.as_ref() {
                    Ok(_) => {
                        tracing::debug!(
                            "using url: {:?}, req: {:?}, res: {:?}",
                            v.url(),
                            req_clone,
                            res
                        );
                    }
                    // this is not a critical error, until all rpc failed.
                    Err(_) => {
                        tracing::warn!(
                            "using url: {:?}, req: {:?}, res: {:?}",
                            v.url(),
                            req_clone,
                            res
                        );
                    }
                }

                res
            }
            .boxed()
        });

        select_ok(res).map_ok(|v| v.0).boxed()
    }
}
