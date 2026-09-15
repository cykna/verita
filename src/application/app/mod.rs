use arboard::Clipboard;
mod invites;
mod messages;
pub(crate) mod notifications;
use common::Sender;

use crate::{
    App,
    application::{Application, services::ApplicationService},
    connection::RequestToConnection,
};

impl<S: ApplicationService> Application<S> {
    pub fn build_window(
        res: Sender<RequestToConnection>,
        clipboard: std::sync::Arc<std::sync::RwLock<Clipboard>>,
    ) -> color_eyre::Result<App> {
        let app = App::new()?;
        messages::setup_send_message(&app, res.clone());
        notifications::setup_notifications(&app);
        invites::setup_find_invite(&app, res.clone());
        invites::setup_request_invite(&app, res, clipboard.clone());
        Ok(app)
    }
}
