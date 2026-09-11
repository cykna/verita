LUCIDE_PATH:= $(shell cargo build 2>&1 | grep "LUCIDE ICON PATH" | sed 's/.*LUCIDE ICON PATH: "//' | sed 's/"$$//')

preview:
	slint-viewer ui/main.slint --auto-reload -L "lucide=$(LUCIDE_PATH)"

migrate-up:
	sea-orm-cli migrate -u sqlite://app.db?mode=rwc up

migrate-down:
	sea-orm-cli migrate -u sqlite://app.db?mode=rwc down
