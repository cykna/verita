use common::Sender;
use slint::{ComponentHandle, Model, ModelExt, ModelRc, VecModel};

use crate::{App, connection::RequestToConnection};

pub(crate) fn setup_send_message(app: &App, res: Sender<RequestToConnection>) {
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
