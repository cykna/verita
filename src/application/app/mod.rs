use arboard::Clipboard;
mod invites;
mod messages;
pub(crate) mod notifications;
mod qrcode;
use common::Sender;
use sea_orm::DatabaseConnection;

use crate::{
    App,
    application::{Application, services::ApplicationService},
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
        invites::setup_find_invite(&app, res.clone());
        invites::setup_request_invite(&app, res, conn.clone(), clipboard.clone());
        Ok(app)
    }
}
