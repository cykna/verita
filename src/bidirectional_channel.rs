pub trait Message: Sized {
    type Response: Message<Response = Self>;
}

///A channel that will be able to send the given `Msg` type, and read its `Response` type.
pub struct Channel<Msg: Message> {
    requester: flume::Sender<Msg>,
    listener: flume::Receiver<Msg::Response>,
}

impl<Msg: Message> Channel<Msg> {
    ///Creates 2 channels that are interconnected. The first is able to send a message and read responses, and the second able to send a response, and read messages
    pub fn new() -> (Channel<Msg>, Channel<Msg::Response>) {
        let (first_requester, first_listener) = flume::unbounded();
        let (second_requester, second_listener) = flume::unbounded();
        (
            Channel {
                requester: first_requester,
                listener: second_listener,
            },
            Channel {
                requester: second_requester,
                listener: first_listener,
            },
        )
    }

    ///Requests the given `content` to the channel connected to this, expecting a response back.
    pub async fn request(&self, content: Msg) -> color_eyre::Result<Msg::Response> {
        self.fire(content).await?;
        let response = self.listener.recv_async().await?;
        Ok(response)
    }

    ///Sends the given `content` to the channel connected to this, and doesnt expect anything back. This is called fire because its fire-n-forget
    pub async fn fire(&self, content: Msg) -> color_eyre::Result<()> {
        if let Err(error) = self.requester.send_async(content).await {
            return Err(color_eyre::Report::msg(format!(
                "Could not send the content through the channel. The other side might be dead.\nFlume Error: {}",
                error.to_string()
            )));
        }
        Ok(())
    }

    pub async fn recv(&self) -> color_eyre::Result<Msg::Response> {
        self.listener
            .recv_async()
            .await
            .map_err(color_eyre::Report::new)
    }
}

impl<M: Message> Clone for Channel<M> {
    fn clone(&self) -> Self {
        Self {
            requester: self.requester.clone(),
            listener: self.listener.clone(),
        }
    }
}
