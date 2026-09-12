use arboard::Clipboard;
mod invites;
use common::Sender;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{
    App,
    application::{Application, services::ApplicationService},
    connection::RequestToConnection,
};

impl<S: ApplicationService> Application<S> {
    fn setup_send_message(app: &App, res: Sender<RequestToConnection>) {
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
    }

    pub fn build_window(
        res: Sender<RequestToConnection>,
        clipboard: std::sync::Arc<std::sync::RwLock<Clipboard>>,
    ) -> color_eyre::Result<App> {
        let app = App::new()?;
        Self::setup_send_message(&app, res.clone());
        invites::setup_find_invite(&app, res.clone());
        invites::setup_request_invite(&app, res, clipboard.clone());
        Ok(app)
    }
}
