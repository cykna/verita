use flume::{Receiver, Sender};

pub struct Pipe<T>(Sender<T>, Receiver<T>);

pub enum PipeError<T> {
    Send(flume::SendError<T>),
    Recv(flume::RecvError),
}

impl<T> Pipe<T> {
    pub fn new() -> (Self, Self) {
        let (s1, r1) = flume::unbounded();
        let (s2, r2) = flume::unbounded();

        (Self(s1, r2), Self(s2, r1))
    }

    ///Sends the provided `data` to the other side of this pipe, and waits it to respond
    pub async fn send(&self, data: T) -> Result<T, PipeError<T>> {
        self.0.send_async(data).await.map_err(PipeError::Send)?;
        self.1.recv_async().await.map_err(PipeError::Recv)
    }

    ///This pipe awaits the other pipe to send data to it.
    pub async fn recv(&self) -> Result<T, PipeError<T>> {
        self.1.recv_async().await.map_err(PipeError::Recv)
    }
}
