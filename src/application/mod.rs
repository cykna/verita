mod app;
mod commands;
mod default_service;
mod services;

use arboard::Clipboard;
use common::{Receiver, Sender};
pub use default_service::*;
pub use services::*;

pub use commands::{KademliaAddressesQuantity, RequestToUi, ResponseFromUi};
use libp2p::futures::StreamExt;
use tracing::{error, info};

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use slint::{ComponentHandle, Model, ModelRc, ToSharedString, VecModel, Weak};

use crate::{
    App, MessageData, MessageOwner,
    application::services::ApplicationService,
    connection::{ApplicationConnection, RequestToConnection, ResponseFromConnection},
    domain::{
        invites::DirectInviteRaw,
        kademlia::KademliaRepository,
        subscription::{Subscription, SubscriptionRepository},
    },
};

pub struct Application<Service: ApplicationService + 'static> {
    app: Weak<App>,
    ui_receiver: Receiver<RequestToUi>,
    services: Service,
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

    async fn handle_request(&mut self, req: RequestToUi) -> ResponseFromUi {
        match req {
            RequestToUi::ReceivedMessage(message) => {
                slint::invoke_from_event_loop({
                    let app = self.app.clone();
                    move || {
                        if let Some(app) = app.upgrade() {
                            app.invoke_send_message(MessageData {
                                content: String::from_utf8_lossy(&message.data).to_shared_string(),
                                owner: MessageOwner::Them,
                            });
                        } else {
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
        let window = Self::build_window(connection_request_channel.clone(), clipboard.clone())?;

        let database = Self::load_database().await?;

        let mut application = Self {
            services: S::new(database, connection_request_channel),
            app: window.as_weak(),
            ui_receiver: ui_listener,
            clipboard,
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
