pub trait Message: Sized {
    type Response: Message<Response = Self>;
}

pub struct ChannelContent<T: Message> {
    pub msg: T,
    pub responder: Option<flume::Sender<T::Response>>,
}

///A channel that will be able to send the given `Msg` type, and read its `Response` type.
pub struct Sender<Msg: Message> {
    requester: flume::Sender<ChannelContent<Msg>>,
}

pub struct Receiver<Msg: Message> {
    listener: flume::Receiver<ChannelContent<Msg>>,
}

pub fn channel<T: Message>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::unbounded();

    (Sender { requester: tx }, Receiver { listener: rx })
}

impl<Msg: Message> Sender<Msg> {
    ///Requests the given `content` to the channel connected to this, expecting a response back.
    pub async fn request(&self, content: Msg) -> color_eyre::Result<Msg::Response> {
        let (tx, rx) = flume::bounded(1);
        if let Err(e) = self
            .requester
            .send_async(ChannelContent {
                msg: content,
                responder: Some(tx),
            })
            .await
        {
            return Err(color_eyre::Report::msg(format!(
                "Could not send the content through the channel. The other side might be dead.\nFlume Error: {e}",
            )));
        };
        let content = rx.recv_async().await?;
        Ok(content)
    }

    ///Sends the given `content` to the channel connected to this, and doesnt expect anything back. This is called fire because its fire-n-forget
    pub async fn fire(&self, content: Msg) -> color_eyre::Result<()> {
        if let Err(error) = self
            .requester
            .send_async(ChannelContent {
                msg: content,
                responder: None,
            })
            .await
        {
            return Err(color_eyre::Report::msg(format!(
                "Could not send the content through the channel. The other side might be dead.\nFlume Error: {}",
                error.to_string()
            )));
        }
        Ok(())
    }
}

impl<T: Message> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            requester: self.requester.clone(),
        }
    }
}
impl<T: Message> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            listener: self.listener.clone(),
        }
    }
}
impl<Msg: Message> Receiver<Msg> {
    pub async fn recv(&self) -> color_eyre::Result<(Msg, Option<flume::Sender<Msg::Response>>)> {
        let content = self.listener.recv_async().await?;
        Ok((content.msg, content.responder))
    }
}
