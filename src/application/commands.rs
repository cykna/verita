use crate::bidirectional_channel::Message;

#[derive(Debug)]
pub enum RequestToUi {
    ReceivedMessage(libp2p::gossipsub::Message),
}
pub enum ResponseFromUi {
    Empty,
}

impl Message for RequestToUi {
    type Response = ResponseFromUi;
}
impl Message for ResponseFromUi {
    type Response = RequestToUi;
}
