use libp2p::Multiaddr;

use crate::bidirectional_channel::Message;

#[derive(Debug)]
pub enum KademliaAddressesQuantity {
    N(u32),
    All,
}

#[derive(Debug)]
pub enum RequestToUi {
    ReceivedMessage(libp2p::gossipsub::Message),
    GetKademliaAddresses(KademliaAddressesQuantity),
}

#[derive(Debug)]
pub enum ResponseFromUi {
    KademliaAddresses(Vec<Multiaddr>),
    Empty,
}

impl Message for RequestToUi {
    type Response = ResponseFromUi;
}
impl Message for ResponseFromUi {
    type Response = RequestToUi;
}
