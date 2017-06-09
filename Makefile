WORKSPACES="./"

lint:
	@- $(foreach WORKSPACE,$(WORKSPACES), \
		cd $(WORKSPACE) ;\
		rustup default stable && cargo fmt -- --write-mode=diff ;\
		rustup default nightly && cargo clippy ;\
		rustup default stable ;\
	)

.PHONY: lint
