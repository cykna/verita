use arboard::Clipboard;

use common::Sender;

use slint::ToSharedString;
use tracing::{error, info};

use crate::{
    App, Invite,
    connection::{RequestToConnection, ResponseFromConnection},
    domain::invites::DirectInviteRaw,
};
pub(crate) fn setup_request_invite(
    app: &App,
    res: Sender<RequestToConnection>,
    clipboard: std::sync::Arc<std::sync::RwLock<Clipboard>>,
) {
    app.on_request_invite({
        move |duration, password| {
            let res = res.clone();
            let clipboard = clipboard.clone();
            tokio::spawn(async move {
                let invite = match res
                    .request(RequestToConnection::GenerateInvite(
                        std::time::Duration::from_secs(duration as u64),
                    ))
                    .await
                {
                    Ok(ResponseFromConnection::Invite(invite)) => invite,
                    Ok(e) => return Err(error!("Invalid response {e:?}")),
                    Err(e) => {
                        return Err(error!("Internal error during request for invite: {e}"));
                    }
                };
                let private_key = match res.request(RequestToConnection::GrantPrivateKey).await {
                    Ok(ResponseFromConnection::PrivateKey(key)) => key,
                    Ok(e) => return Err(error!("Invalid response {e:?}")),
                    Err(e) => {
                        return Err(error!("Internal error during request for invite: {e}"));
                    }
                };

                let raw_invite = {
                    let temp = invite
                        .as_raw(&private_key, password.as_bytes())
                        .expect("Couldn't generate raw invite");
                    postcard::to_allocvec(&temp).unwrap()
                };
                let bs58_invite = bs58::encode(raw_invite).into_string();

                {
                    let mut lock = clipboard.write().unwrap();
                    info!("Writing into clipboard");
                    lock.set_text(bs58_invite)
                        .expect("Should be able to store the invite on clipboard");
                    info!(
                        "Successfully wrote on clipboard {}",
                        lock.get_text().unwrap()
                    );
                }

                Ok(())
            });
        }
    });
}

pub(crate) fn error_invite() -> crate::Invite {
    let empty = "".to_shared_string();
    crate::Invite {
        address: empty.clone(),
        peer: empty.clone(),
        valid: false,
        timestamp: 0,
    }
}

pub fn found_invite(invite: Invite, error: Option<color_eyre::Report>) -> crate::FoundInvite {
    crate::FoundInvite {
        invite,
        error: error.map(|e| e.to_shared_string()).unwrap_or_default(),
    }
}

pub(crate) fn setup_find_invite(app: &App, res: Sender<RequestToConnection>) {
    app.on_find_invite({
        move |invite, password| {
            let raw_invite = match bs58::decode(invite.as_str()).into_vec() {
                Ok(raw) => raw,
                Err(e) => {
                    error!("{}", e);
                    return found_invite(error_invite(), Some(e.into()));
                }
            };
            let invite = match postcard::from_bytes::<DirectInviteRaw>(&raw_invite) {
                Ok(invite) => invite.as_direct(password.as_bytes()),
                Err(e) => Err(e.into()),
            };

            match invite {
                Ok(invite) => {
                    info!("I found it. {}", invite.metadata().address.to_string());
                    found_invite(
                        crate::Invite {
                            address: invite.metadata().address.to_shared_string(),
                            peer: invite.metadata().peer.to_shared_string(),
                            valid: true,
                            timestamp: invite.metadata().timestamp as i32,
                        },
                        None,
                    )
                }
                Err(e) => found_invite(error_invite(), Some(e.into())),
            }
        }
    });
}
