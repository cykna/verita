use crate::{bidirectional_channel::Message, domain::subscription::Subscription};

#[derive(Debug)]
pub enum RequestToConnection {
    SendMessage(String),
    ///Joins the topic with the given id
    JoinTopic(Subscription),
}
pub enum ResponseFromConnection {
    Empty,
    Error(color_eyre::Report),
}

impl Message for RequestToConnection {
    type Response = ResponseFromConnection;
}
impl Message for ResponseFromConnection {
    type Response = RequestToConnection;
}
