use tokio::sync::{mpsc, oneshot};

pub struct Request<Msg, Resp> {
    pub msg: Msg,
    pub resp: oneshot::Sender<Resp>,
}

pub fn channel<Msg, Resp>(
    buffer: usize,
) -> (
    mpsc::Sender<Request<Msg, Resp>>,
    mpsc::Receiver<Request<Msg, Resp>>,
) {
    mpsc::channel(buffer)
}
