use super::uri::URI;
use crate::errors::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::SetupPayload;
use crate::spi::{EmptyRSocket, RSocket};
use crate::transport::{
    Acceptor, ClientTransport, DuplexSocket, FnAcceptorWithSetup, ServerTransport,
};
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

type FnStart = fn();

pub struct ServerBuilder<T, C>
where
    T: Send + Sync + ServerTransport<Item = C>,
    C: Send + Sync + ClientTransport,
{
    transport: Option<T>,
    on_setup: FnAcceptorWithSetup,
    start_handler: Option<FnStart>,
}

impl<T, C> ServerBuilder<T, C>
where
    T: Send + Sync + ServerTransport<Item = C> + 'static,
    C: Send + Sync + ClientTransport + 'static,
{
    pub(crate) fn new() -> ServerBuilder<T, C> {
        ServerBuilder {
            transport: None,
            on_setup: on_setup_noop,
            start_handler: None,
        }
    }

    pub fn acceptor(mut self, handler: FnAcceptorWithSetup) -> Self {
        self.on_setup = handler;
        self
    }

    pub fn on_start(mut self, hanlder: FnStart) -> Self {
        self.start_handler = Some(hanlder);
        self
    }

    pub fn transport(mut self, transport: T) -> Self {
        self.transport = Some(transport);
        self
    }

    pub async fn serve(mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.transport.take() {
            None => panic!("missing transport"),
            Some(tp) => {
                tp.start(self.start_handler, move |tp| {
                    let setuper = Arc::new(self.on_setup);
                    let (rcv_tx, rcv_rx) = mpsc::unbounded_channel::<Frame>();
                    let (snd_tx, snd_rx) = mpsc::unbounded_channel::<Frame>();
                    tokio::spawn(async move {
                        tp.attach(rcv_tx, snd_rx).await.unwrap();
                    });
                    tokio::spawn(async move {
                        let ds = DuplexSocket::new(0, snd_tx).await;
                        let acceptor = Acceptor::Generate(setuper.clone());
                        ds.event_loop(acceptor, rcv_rx).await;
                    });
                })
                .await
            }
        }
    }
}

#[inline]
fn on_setup_noop(
    _setup: SetupPayload,
    _socket: Box<dyn RSocket>,
) -> Result<Box<dyn RSocket>, Box<dyn Error>> {
    Ok(Box::new(EmptyRSocket))
}
