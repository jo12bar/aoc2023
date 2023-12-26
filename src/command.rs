use futures::prelude::*;
use tokio::sync::mpsc::Sender;

pub enum Cmd<Msg> {
    None,
    Future(future::BoxFuture<'static, Msg>),
    Msg(Msg),
}

impl<Msg> Cmd<Msg> {
    pub fn boxed<F>(f: F) -> Self
    where
        F: Future<Output = Msg> + Send + 'static,
    {
        Self::Future(Box::pin(f))
    }
}

pub fn process_cmd<Msg: Send + 'static>(cmd: Cmd<Msg>, msg_tx: Sender<Msg>) {
    match cmd {
        Cmd::Future(fut) => {
            tokio::spawn(async move {
                let msg = fut.await;
                if let Err(e) = msg_tx.send(msg).await {
                    panic!("failed to send message from asynchronous command due to closed channel: {e}");
                }
            });
        },

        Cmd::Msg(msg) => {
            tokio::spawn(async move {
                if let Err(e) = msg_tx.send(msg).await {
                    panic!("failed to send message from synchronous command due to closed channel: {e}");
                }
            });
        },

        Cmd::None => {},
    }
}
