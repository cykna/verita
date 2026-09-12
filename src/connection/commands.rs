use std::time::Duration;

use common::Message;

use crate::domain::{invites::DirectInvite, subscription::Subscription};

#[derive(Debug)]
pub enum RequestToConnection {
    SendMessage(String),
    ///Joins the topic with the given id
    JoinTopic(Subscription),
    ///Creates an invite that will be kept alive until the given timestamp
    GenerateInvite(Duration),
    GrantPrivateKey,
}

#[derive(Debug)]
pub enum ResponseFromConnection {
    None,
    Invite(DirectInvite),
    PrivateKey([u8; 32]),
    Error(color_eyre::Report),
}

impl Message for RequestToConnection {
    type Response = ResponseFromConnection;
}
impl Message for ResponseFromConnection {
    type Response = RequestToConnection;
}
