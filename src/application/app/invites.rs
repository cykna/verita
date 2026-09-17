use arboard::Clipboard;

use color_eyre::eyre::eyre;
use common::Sender;

use sea_orm::DatabaseConnection;
use slint::{ComponentHandle, ToSharedString, Weak};
use tracing::{error, info};

use crate::{
    App, NotificationData,
    application::app::notifications::notify_data,
    connection::{RequestToConnection, ResponseFromConnection},
    domain::invites::{
        network::DirectInviteRaw,
        repository::{CreateInviteDescriptor, InvitesRepository},
    },
};

pub(crate) fn error_invite() -> crate::Invite {
    let empty = "".to_shared_string();
    crate::slint_generatedApp::Invite {
        address: empty.clone(),
        peer: empty.clone(),
        valid: false,
        timestamp: 0,
    }
}

pub fn found_invite(
    invite: crate::Invite,
    error: Option<color_eyre::Report>,
) -> crate::FoundInvite {
    crate::FoundInvite {
        invite,
        error: error.map(|e| e.to_shared_string()).unwrap_or_default(),
    }
}

pub fn exec_notifying_async(
    app: Weak<App>,
    f: impl Future<Output = color_eyre::Result> + Send + 'static,
) -> color_eyre::Result {
    slint::invoke_from_event_loop(move || {
        slint::spawn_local(async move {
            if let Err(e) = f.await
                && let Some(app) = app.upgrade()
            {
                notify_data(
                    &app,
                    NotificationData::new(
                        "Error",
                        e.to_string(),
                        std::time::Duration::from_secs(3),
                    ),
                );
                error!("Error during operation: {}", e);
            }
        })
        .unwrap();
    })?;
    Ok(())
}
///Setups the callback 'request-invite'. This callback is intended to retrieve a new invite with the informations of the user, and then, paste it into his clipboard.
///This callback will internally:
/// * Generate a new invite and set it into the clipboard
/// * Write a new local invite into the user db.
pub(crate) fn setup_request_invite(
    app: &App,
    res: Sender<RequestToConnection>,
    conn: DatabaseConnection,
    clipboard: std::sync::Arc<std::sync::RwLock<Clipboard>>,
) {
    app.global::<crate::Callbacks>().on_request_invite({
        let app = app.as_weak();
        move |duration, password| {
            let conn = conn.clone();
            let res = res.clone();
            let clipboard = clipboard.clone();
            let app = app.clone();
            let _ = exec_notifying_async(app.clone(), async move {
                let invite = match res
                    .request(RequestToConnection::GenerateInvite(
                        std::time::Duration::from_secs(duration as u64),
                    ))
                    .await
                {
                    Ok(ResponseFromConnection::Invite(invite)) => invite,
                    Ok(e) => return Err(eyre!("Invalid response {e:?}")),
                    Err(e) => {
                        return Err(eyre!("Internal error during request for invite: {e}"));
                    }
                };
                let private_key = match res.request(RequestToConnection::GrantPrivateKey).await {
                    Ok(ResponseFromConnection::PrivateKey(key)) => key,
                    Ok(e) => return Err(eyre!("Invalid response {e:?}")),
                    Err(e) => {
                        return Err(eyre!("Internal error during request for invite: {e}",));
                    }
                };

                let hash_invite = conn
                    .write_invite(CreateInviteDescriptor {
                        invite: invite,
                        private_key: private_key,
                        password: password.to_string().into_bytes(),
                        max_usage: 12,
                    })
                    .await?;
                {
                    let mut lock = clipboard.write().unwrap();
                    lock.set_text(&hash_invite.0)?;
                }
                info!("Copying '{}' into the clipboard", hash_invite.0);
                //conn.;
                notify_data(
                    &app.upgrade().unwrap(),
                    NotificationData::new_with_default_duration(
                        "Success",
                        "Successfully created the invite and wrote it to your clipboard",
                    ),
                );

                Ok(())
            });
        }
    });
}

pub(crate) fn setup_find_invite(app: &App, _: Sender<RequestToConnection>) {
    app.global::<crate::Callbacks>().on_find_invite({
        move |invite, password| {
            let raw_invite = match bs58::decode(invite.as_str()).into_vec() {
                Ok(raw) => raw,
                Err(e) => {
                    error!("{}", e);
                    return found_invite(error_invite(), Some(e.into()));
                }
            };
            let invite = match postcard::from_bytes::<DirectInviteRaw>(&raw_invite) {
                Ok(invite) => invite.retrieve_direct(password.as_bytes()),
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
                Err(e) => found_invite(error_invite(), Some(e)),
            }
        }
    });
}
