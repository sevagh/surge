WORKSPACES="./" "./app_setup"

lint:
	@- $(foreach WORKSPACE,$(WORKSPACES), \
		cd $(WORKSPACE) ;\
		cargo +nightly fmt;\
		cargo +nightly clippy;\
	)

.PHONY: lint
