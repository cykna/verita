use std::collections::HashMap;

use common::Message;
use libp2p::{Multiaddr, PeerId};

<<<<<<< HEAD
use crate::NotificationData;

=======
>>>>>>> 832a380 (chore: updated usage of channel to sender/receiver)
pub type KademliaAddressesQuantity = Option<u64>;

#[derive(Debug)]
pub enum RequestToUi {
    ReceivedMessage(libp2p::gossipsub::Message),
    ///Requests the Kademlia addresses of the peers in the network
    GetKademliaAddresses(KademliaAddressesQuantity),
    Notify(NotificationData),
}

#[derive(Debug)]
pub enum ResponseFromUi {
    ///Returns the Kademlia addresses of the peers in the network.
    KademliaAddresses(HashMap<PeerId, Vec<Multiaddr>>),
    Empty,
}

impl Message for RequestToUi {
    type Response = ResponseFromUi;
}
impl Message for ResponseFromUi {
    type Response = RequestToUi;
}
