use arboard::Clipboard;
mod invites;
mod messages;
pub(crate) mod notifications;
use common::Sender;
use sea_orm::DatabaseConnection;
use slint::Weak;

use crate::{
    App,
    application::{Application, app::notifications::notify_data, services::ApplicationService},
    connection::RequestToConnection,
};

impl<S: ApplicationService> Application<S> {
    pub fn build_window(
        res: Sender<RequestToConnection>,
        conn: DatabaseConnection,
        clipboard: std::sync::Arc<std::sync::RwLock<Clipboard>>,
    ) -> color_eyre::Result<App> {
        let app = App::new()?;
        messages::setup_send_message(&app, res.clone());
        notifications::setup_notifications(&app);
        invites::setup_find_local_invites(&app, res.clone(), conn.clone());
        invites::setup_find_invite(&app, res.clone(), conn.clone());
        invites::setup_request_invite(&app, res, conn.clone(), clipboard.clone());
        Ok(app)
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
                    crate::NotificationData::new(
                        "Error",
                        e.to_string(),
                        std::time::Duration::from_secs(3),
                    ),
                );
                tracing::error!("Error during operation: {}", e);
            }
        })
        .unwrap();
    })?;
    Ok(())
}
