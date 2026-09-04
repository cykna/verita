use libp2p::futures::StreamExt;
use tracing::{error, info};

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use slint::{ComponentHandle, Model, ModelRc, ToSharedString, VecModel, Weak};

use crate::{
    App, MessageData, MessageOwner,
    bidirectional_channel::{Channel, Message},
    connection::{ApplicationConnection, RequestToConnection},
    model::subscriptions::Subscription,
    repositories::Repository,
};

pub struct Application {
    app: Weak<App>,
    connection_sender: Channel<RequestToConnection>,
    ui_receiver: Channel<ResponseFromUi>,
    database: DatabaseConnection,
}

pub enum RequestToUi {
    ReceivedMessage(libp2p::gossipsub::Message),
} //req to ui
pub enum ResponseFromUi {
    Empty,
}

impl Message for RequestToUi {
    type Response = ResponseFromUi;
}
impl Message for ResponseFromUi {
    type Response = RequestToUi;
}

impl Application {
    pub async fn join_topic(&self, topic: String, persist: bool) -> color_eyre::Result<()> {
        if persist {
            self.database
                .insert(Subscription { id: topic.clone() })
                .await?;
        }
        self.connection_sender
            .fire(RequestToConnection::JoinTopic(topic))
            .await
    }

    pub async fn setup(&mut self) -> color_eyre::Result<()> {
        for subscription in self.database.find_all().await? {
            self.join_topic(subscription.id, false).await?;
        }
        self.join_topic("hello".into(), true).await?;
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
                    if let Ok(_) = res
                        .request(RequestToConnection::SendMessage(
                            message.content.to_string(),
                        ))
                        .await
                    {
                        let res = slint::invoke_from_event_loop(move || {
                            if let Some(app) = app.upgrade() {
                                let messages = app.get_messages().iter().collect::<VecModel<_>>();
                                messages.push(message);
                                app.set_messages(ModelRc::new(messages));
                            }
                        });
                    } else {
                    }
                });
            }
        });
        Ok(app)
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
                            info!("Invokated received message wow");
                        } else {
                            error!("Conseguiu o upgrade não bixo");
                        }
                    }
                });
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
            database,
            app: window.as_weak(),
            ui_receiver: ui_response_channel,
            connection_sender: connection_request_channel,
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
