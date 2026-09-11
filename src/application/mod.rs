mod app;
mod commands;
mod default_service;
mod services;

<<<<<<< HEAD
use arboard::Clipboard;
use common::Receiver;
=======
use common::{Receiver, Sender};
>>>>>>> 832a380 (chore: updated usage of channel to sender/receiver)
pub use default_service::*;
pub use services::*;

pub use commands::{KademliaAddressesQuantity, RequestToUi, ResponseFromUi};
use libp2p::futures::StreamExt;
use tracing::error;

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use slint::{ComponentHandle, ToSharedString, Weak};

use crate::{
    App, MessageData, MessageOwner,
<<<<<<< HEAD
    application::{app::notifications::notify, services::ApplicationService},
    connection::{ApplicationConnection, ResponseFromConnection},
=======
    application::services::ApplicationService,
    connection::{
        ApplicationConnection, InviteResponse, RequestToConnection, ResponseFromConnection,
    },
>>>>>>> 832a380 (chore: updated usage of channel to sender/receiver)
    domain::{
        kademlia::KademliaRepository,
        subscription::{Subscription, SubscriptionRepository},
    },
};

pub struct Application<Service: ApplicationService + 'static> {
    app: Weak<App>,
    ui_receiver: Receiver<RequestToUi>,
    services: Service,
    #[allow(dead_code)]
    clipboard: std::sync::Arc<std::sync::RwLock<Clipboard>>,
}

impl<S: ApplicationService> Application<S> {
    pub async fn join_topic(&self, topic: Subscription, persist: bool) -> color_eyre::Result<()> {
        if persist {
            self.services
                .subscription_repo()
                .upsert(topic.clone())
                .await?;
        }
        self.services
            .connection_requester()
            .join_topic(&topic.id)
            .await
    }

    pub async fn setup(&mut self) -> color_eyre::Result<()> {
        for subscription in self.services.subscription_repo().find_all().await? {
            self.join_topic(subscription, false).await?;
        }
        self.join_topic(Subscription { id: "hello".into() }, true)
            .await?;
        Ok(())
    }

    pub async fn load_database() -> color_eyre::Result<DatabaseConnection> {
        let database = Database::connect("sqlite://database/app.db?mode=rwc").await?;
        Migrator::up(&database, None).await?;
        Ok(database)
    }

<<<<<<< HEAD
=======
    pub fn build_window(res: Sender<RequestToConnection>) -> color_eyre::Result<App> {
        let app = App::new()?;
        app.on_send_message({
            let app = app.as_weak();
            let res = res.clone();
            move |message| {
                let app = app.clone();
                let res = res.clone();
                tokio::spawn(async move {
                    let Ok(_) = res
                        .request(RequestToConnection::SendMessage(
                            message.content.to_string(),
                        ))
                        .await
                    else {
                        return;
                    };
                    slint::invoke_from_event_loop(move || {
                        if let Some(app) = app.upgrade() {
                            let messages = app.get_messages().iter().collect::<VecModel<_>>();
                            messages.push(message);
                            app.set_messages(ModelRc::new(messages));
                        }
                    })
                    .unwrap();
                });
            }
        });
        app.on_request_invite({
            let app = app.as_weak();
            let res = res.clone();
            move |duration| {
                let app = app.clone();
                let res = res.clone();
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
                    slint::invoke_from_event_loop(move || {
                        let Some(app) = app.upgrade() else {
                            return;
                        };
                        match invite {
                            InviteResponse::InWait => {
                                let initializing_text = "Initializing yet".to_shared_string();
                                app.set_invite(crate::Invite {
                                    address: initializing_text.clone(),
                                    peer: initializing_text.clone(),
                                    timestamp: 0,
                                    valid: true,
                                });
                            }
                            InviteResponse::Success(invite) => {
                                let metadata = invite.metadata();
                                app.set_invite(crate::Invite {
                                    address: metadata.address.to_shared_string(),
                                    peer: metadata.peer.to_shared_string(),
                                    timestamp: metadata.timestamp as i32,
                                    valid: true,
                                });
                            }
                        }
                    })
                    .unwrap();
                    Ok(())
                });
            }
        });
        Ok(app)
    }

>>>>>>> 832a380 (chore: updated usage of channel to sender/receiver)
    async fn handle_request(&mut self, req: RequestToUi) -> ResponseFromUi {
        match req {
            RequestToUi::ReceivedMessage(message) => {
                slint::invoke_from_event_loop({
                    let app = self.app.clone();
                    move || {
                        if let Some(app) = app.upgrade() {
                            let text = String::from_utf8_lossy(&message.data);
                            app.global::<crate::Callbacks>()
                                .invoke_send_message(MessageData {
                                    content: text.to_shared_string(),
                                    owner: MessageOwner::Them,
                                });
                            notify(
                                &app,
                                "New message",
                                text.to_string(),
                                std::time::Duration::from_secs(3),
                            );
                        }
                    }
                })
                .unwrap();
                ResponseFromUi::Empty
            }
            RequestToUi::GetKademliaAddresses(quantity) => {
                let addresses = self
                    .services
                    .kademlia_repo()
                    .find_addresses(quantity)
                    .await
                    .unwrap();
                ResponseFromUi::KademliaAddresses(addresses)
            }
            RequestToUi::Notify(notfication) => {
                if let Some(app) = self.app.upgrade() {
                    app.global::<crate::Callbacks>().invoke_notify(notfication);
                }
                ResponseFromUi::Empty
            }
        }
    }

    pub async fn run() -> color_eyre::Result<()> {
        let (ui_requester, ui_listener) = common::channel::<RequestToUi>();
        let (connection_request_channel, connection_response_channel) = common::channel();
        let (tx, mut rx) = tokio::sync::broadcast::channel::<()>(2);

        tokio::spawn({
            let mut swarm = ApplicationConnection::new(ui_requester).await?;

            let mut rx = tx.subscribe();
            let tx = tx.clone();
            async move {
                if let Err(e) = swarm.setup().await {
                    error!("Error while setup: {e}");
                    tx.send(())?;
                    return Err(e);
                };
                loop {
                    tokio::select! {
                        Ok(_) = rx.recv() => {
                            break;
                        }
                        event = swarm.select_next_some() => if let Err(e) = swarm.handle_event(event).await {
                            error!("{e:?}");
                            continue;
                        },
                        Ok((req, responder)) = connection_response_channel.recv() => {
                            let response = match swarm.handle_request(req).await {
                                Ok(res) => res,
                                Err(e) => {
                                    error!("Error during request handling: '{e:?}'");
                                    ResponseFromConnection::Error(e)
                                }
                            };

                            if let Some(responder) = responder && let Err(e) = responder.send_async(response).await {
                                error!("Couldnt fire the connection response back: '{e:?}'");
                            }
                        }

                    }
                }
                Ok::<(), color_eyre::Report>(())
            }
        });
        let clipboard = std::sync::Arc::new(std::sync::RwLock::new(Clipboard::new().unwrap()));
        let database = Self::load_database().await?;
        let services = S::new(database.clone(), connection_request_channel.clone());
        let window = Self::build_window(
            services.connection_requester().clone(),
            database.clone(),
            clipboard.clone(),
        )?;

        let mut application = Self {
            services,
            app: window.as_weak(),
            ui_receiver: ui_listener,
<<<<<<< HEAD
            clipboard,
=======
>>>>>>> 832a380 (chore: updated usage of channel to sender/receiver)
        };
        application.setup().await?;
        tokio::spawn({
            async move {
                loop {
                    tokio::select! {
                        Ok(_) = rx.recv() => {
                            slint::quit_event_loop().unwrap();
                            break;
                        },
                        Ok((req, responder)) = application.ui_receiver.recv() => {
                            let response = application.handle_request(req).await;
                            if let Some(responder) = responder && let Err(e) = responder.send_async(response).await {
                                error!("{e}");
                                break;
                            }
                        }
                    }
                }
            }
        });
        window.run()?;
        tx.send(()).unwrap();
        Ok(())
    }
}
