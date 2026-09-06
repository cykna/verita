mod commands;
mod default_service;
mod services;

pub use default_service::*;
pub use services::*;

pub use commands::*;
use libp2p::futures::StreamExt;
use tracing::{error, info};

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use slint::{ComponentHandle, Model, ModelRc, ToSharedString, VecModel, Weak};

use crate::{
    App, MessageData, MessageOwner,
    application::{commands::ResponseFromUi, services::ApplicationService},
    bidirectional_channel::Channel,
    connection::{ApplicationConnection, RequestToConnection},
    domain::subscription::{Subscription, SubscriptionRepository},
};

pub struct Application<Service: ApplicationService + 'static> {
    app: Weak<App>,
    ui_receiver: Channel<ResponseFromUi>,
    services: Service,
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

    pub fn build_window(res: Channel<RequestToConnection>) -> color_eyre::Result<App> {
        let app = App::new()?;
        app.on_send_message({
            let app = app.as_weak();
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
        Ok(app)
    }

    async fn handle_request(&mut self, req: RequestToUi) -> ResponseFromUi {
        info!("Application received request {req:?}");
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
                            info!("Invokated received message wow");
                        } else {
                            error!("Conseguiu o upgrade não bixo");
                        }
                    }
                })
                .unwrap();
            }
        }
        ResponseFromUi::Empty
    }

    pub async fn run() -> color_eyre::Result<()> {
        let (ui_request_channel, ui_response_channel) = Channel::new();
        let (connection_request_channel, connection_response_channel) = Channel::new();
        let (tx, mut rx) = tokio::sync::broadcast::channel::<()>(2);

        tokio::spawn({
            let mut swarm = ApplicationConnection::new(ui_request_channel)?;
            let mut rx = tx.subscribe();

            async move {
                loop {
                    tokio::select! {
                        Ok(_) = rx.recv() => {
                            break;
                        }
                        event = swarm.select_next_some() => if let Err(e) = swarm.handle_event(event).await {
                            error!("{e:?}");
                            continue;
                        },
                        Ok(req) = connection_response_channel.recv() => {
                            let response = match swarm.handle_request(req).await {
                                Ok(res) => res,
                                Err(e) => {
                                    error!("{e:?}");
                                    continue;
                                }
                            };
                            if let Err(e) = connection_response_channel.fire(response).await {
                                error!("{e:?}");
                            }
                        }

                    }
                }
            }
        });
        let window = Self::build_window(connection_request_channel.clone())?;
        let database = Self::load_database().await?;
        let mut application = Self {
            services: S::new(database, connection_request_channel),
            app: window.as_weak(),
            ui_receiver: ui_response_channel,
        };
        application.setup().await?;
        tokio::spawn({
            async move {
                loop {
                    tokio::select! {
                        Ok(_) = rx.recv() => break,
                        Ok(req) = application.ui_receiver.recv() => {
                            let response = application.handle_request(req).await;
                            if let Err(e) = application.ui_receiver.fire(response).await {
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
