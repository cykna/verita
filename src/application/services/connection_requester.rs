use crate::{
    bidirectional_channel::Channel, connection::RequestToConnection,
    domain::subscription::Subscription,
};

pub struct ConnectionRequester {
    channel: Channel<RequestToConnection>,
}

impl ConnectionRequester {
    pub fn new(channel: Channel<RequestToConnection>) -> Self {
        Self { channel }
    }
    pub async fn join_topic(&self, topic_id: &str) -> color_eyre::Result<()> {
        self.channel
            .fire(RequestToConnection::JoinTopic(Subscription {
                id: topic_id.to_string(),
            }))
            .await
    }
}
