use std::{
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};

use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{App, NotificationData};

static NOTIFICATION_KEY: AtomicI32 = AtomicI32::new(0);

impl NotificationData {
    pub fn default_duration() -> std::time::Duration {
        std::time::Duration::from_millis(3000)
    }

    pub fn new(
        title: impl Into<SharedString>,
        text: impl Into<SharedString>,
        duration: std::time::Duration,
    ) -> Self {
        Self {
            duration: duration.as_millis() as i64,
            key: NOTIFICATION_KEY.fetch_add(1, Ordering::Relaxed),
            title: title.into(),
            text: text.into(),
        }
    }
    pub fn new_with_default_duration(
        title: impl Into<SharedString>,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            duration: Self::default_duration().as_millis() as i64,
            key: NOTIFICATION_KEY.fetch_add(1, Ordering::Relaxed),
            title: title.into(),
            text: text.into(),
        }
    }
}

///Emits a new notification from outside the UI event loop. New notifications
///are stacked below the already visible ones and auto-expire after `duration`.
pub fn notify(
    app: &App,
    title: impl Into<SharedString>,
    text: impl Into<SharedString>,
    duration: std::time::Duration,
) {
    notify_data(app, NotificationData::new(title, text, duration));
}

///Emits a notification already built by the caller.
pub fn notify_data(app: &App, notification: NotificationData) {
    if let Some(app) = app.as_weak().upgrade() {
        app.global::<crate::Callbacks>().invoke_notify(notification);
    }
}

pub(crate) fn setup_notifications(app: &App) {
    let notifications = Rc::new(VecModel::<NotificationData>::default());
    app.set_notifications(ModelRc::new(notifications.clone()));
    app.global::<crate::Callbacks>().on_notify({
        let notifications = notifications.clone();
        move |notification| {
            notifications.push(notification);
        }
    });

    app.global::<crate::Callbacks>().on_notify_end({
        let notifications = notifications.clone();
        move |key| {
            for index in (0..notifications.row_count()).rev() {
                if notifications
                    .row_data(index)
                    .is_some_and(|notification| notification.key == key)
                {
                    notifications.remove(index);
                    break;
                }
            }
        }
    });

    app.global::<crate::Callbacks>()
        .on_retrieve_notification_for(|title, text, duration| {
            NotificationData::new(
                title,
                text,
                std::time::Duration::from_millis(duration as u64),
            )
        });
}
