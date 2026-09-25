use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::{
    App,
    application::{ConnectionRequester, app::exec_notifying_async},
};

pub(crate) fn setup_send_message(app: &App, res: ConnectionRequester) {
    app.global::<crate::Callbacks>().on_send_message({
        let app = app.as_weak();
        move |message| {
            let app = app.clone();
            let res = res.clone();
            let _ = exec_notifying_async(app.clone(), async move {
                res.send_message(message.content.to_string()).await?;

                if let Some(app) = app.upgrade() {
                    let messages = app.get_messages().iter().collect::<VecModel<_>>();
                    messages.push(message);
                    app.set_messages(ModelRc::new(messages));
                }

                Ok(())
            });
        }
    });
}
