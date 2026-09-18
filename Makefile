LUCIDE_PATH := $(shell \
	if [ -f .lucide_path ]; then \
		cat .lucide_path; \
	else \
		cargo build 2>&1 | grep "LUCIDE ICON PATH" | sed 's/.*LUCIDE ICON PATH: "//' | sed 's/"$$//' | tee .lucide_path; \
	fi \
)
ZED_DIR := .zed
ZED_SETTINGS := $(ZED_DIR)/settings.json
.PHONY: setup
setup:
	@mkdir -p $(ZED_DIR)
	@printf '%s\n' \
		'{' \
		'  "lsp": {' \
		'    "slint": {' \
		'      "binary": {' \
		'        "arguments": [' \
		'          "-L",' \
		'          "lucide=$(LUCIDE_PATH)"' \
		'        ]' \
		'      }' \
		'    }' \
		'  }' \
		'}' > $(ZED_SETTINGS)

preview:
	slint-viewer ui/main.slint --auto-reload -L "lucide=$(LUCIDE_PATH)"

migrate-up:
	sea-orm-cli migrate -u sqlite://app.db?mode=rwc up

migrate-down:
	sea-orm-cli migrate -u sqlite://app.db?mode=rwc down
