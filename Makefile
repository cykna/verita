preview:
	slint-viewer ui/main.slint --auto-reload

migrate-up:
	sea-orm-cli migrate -u sqlite://app.db?mode=rwc up

migrate-down:
	sea-orm-cli migrate -u sqlite://app.db?mode=rwc down
