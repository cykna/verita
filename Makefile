

preview:
	slint-viewer ui/main.slint

migrate-up:
	sea-orm-cli migrate -u sqlite://database/app.db?mode=rwc up

migrate-down:
	sea-orm-cli migrate -u sqlite://database/app.db?mode=rwc down
