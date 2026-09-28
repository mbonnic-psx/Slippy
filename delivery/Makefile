# Local and CI entry points. Compatible with GNU Make 3.81 (the macOS default).
SHELL := /bin/bash
.DEFAULT_GOAL := help
INTEGRATION_TEST_SLIPWAI ?= @echo 'slipwai: no integration command recorded in project.json (.)'

.PHONY: help
help: ## Show the available targets
	@grep -hE '^[a-z][a-zA-Z0-9_-]*:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{ printf "  %-22s %s\n", $$1, $$2 }'

.PHONY: install
install: ## Install native dependencies; refresh agent projections after init
	make install
	@if [ -f .specify/integration.json ]; then $(MAKE) -f delivery/Makefile --no-print-directory agents; else echo 'Spec Kit not initialized; run ./delivery/init when ready.'; fi

.PHONY: check-python agents agents-list check-extensions check-agents models check-benchmark benchmark cruise cruise-watch cruise-status cruise-stop cruise-tell check-decisions
check-python: ## Fail, first, when python3 is older than the 3.10 every gate script is written for
	@python3 -c "import sys; sys.exit(0 if sys.version_info >= (3, 10) else 'check-python: ' + sys.executable + ' is Python ' + sys.version.split()[0] + ', and the gate scripts need 3.10 or newer; put a newer python3 first on PATH (a non-interactive macOS shell finds /usr/bin/python3, which is 3.9, before Homebrew)')"
agents: ## Refresh elected extensions, then skills, commands and agent types in every installed agent harness
	python3 delivery/scripts/extensions/project.py
	python3 delivery/scripts/agents/project.py
agents-list: ## Show every supported harness and which integrations are installed
	python3 delivery/scripts/agents/project.py --list
check-extensions: ## Fail when elected extension guidance differs from the factory-owned source
	python3 delivery/scripts/extensions/project.py --check
check-agents: ## Fail when an initialized agent projection has drifted, or .specify/models.json, drive.json or cruise.json is malformed
	python3 delivery/scripts/agents/project.py --check
	python3 delivery/scripts/agents/models.py --check && python3 delivery/scripts/agents/drive.py --check && python3 delivery/scripts/agents/cruise.py --check
models: ## Show which model runs each stage of /drive for the installed harness, and why
	python3 delivery/scripts/agents/models.py
check-benchmark: ## Fail when the benchmark script's own behaviour regresses; warn of an entry left open or a done slice with no record
	python3 delivery/scripts/test_benchmark.py && python3 delivery/scripts/agents/benchmark.py check
benchmark: ## Show what each slice cost and how each stage of /drive did, from the records under specs/
	python3 delivery/scripts/agents/benchmark.py
cruise: ## Run /drive with nobody at the wheel, a fresh session per iteration, until the specs are satisfied or a person stops it (FEATURE=<name> to scope it)
	python3 delivery/scripts/agents/cruise.py run $(if $(FEATURE),--feature $(FEATURE),) $(CRUISE_FLAGS)
cruise-watch: ## Watch a /cruise run from here: what the iteration does as it happens, returning at the iteration's end, a park, or the run's end (CRUISE_FLAGS="--minutes 10" to sit longer)
	python3 delivery/scripts/agents/cruise.py watch $(CRUISE_FLAGS)
cruise-status: ## Say whether a /cruise runner is running and what its log shows: iterations run, the last line, whether it is parked and why
	python3 delivery/scripts/agents/cruise.py status
cruise-stop: ## End a /cruise run after the iteration in flight (CRUISE_FLAGS=--now ends that iteration too)
	python3 delivery/scripts/agents/cruise.py stop $(CRUISE_FLAGS)
cruise-tell: ## Queue a message for the next /cruise iteration (MSG="…"; CRUISE_FLAGS=--now ends the iteration in flight so it goes at once)
	python3 delivery/scripts/agents/cruise.py tell $(CRUISE_FLAGS) $(MSG)
check-decisions: ## Fail when a decision log or demo log /cruise wrote has lost its shape
	python3 delivery/scripts/check-decisions.py

.PHONY: typecheck lint check-imports check-migrations check-slice-scope check-flags check-speckit check-codegraph check-ux-gates check-constitution constitution-requirements
typecheck: ## Run the native compiler or static type check
	python3 delivery/scripts/ratchet.py slipwai typecheck -- 'make typecheck'
lint: ## Run the native formatting and static-analysis gate
	python3 delivery/scripts/ratchet.py slipwai lint -- 'make lint check-structure'

check-imports: ## Fail when domain code imports an outer architecture layer
	python3 delivery/scripts/check-imports.py
check-migrations: ## Fail when a migration contracts the schema without naming the earlier expand it completes
	python3 delivery/scripts/check-migrations.py
check-slice-scope: ## Fail when a slice/<id> branch touches what a sibling slice may also be writing
	python3 delivery/scripts/check-slice-scope.py
check-speckit: ## Fail when an initialized Spec Kit-managed file differs from its manifest
	python3 delivery/scripts/check-speckit.py
check-codegraph: ## Fail when the adopted code index no longer describes the tracked source
	python3 delivery/scripts/check-codegraph.py
check-ux-gates: ## Fail when a browser app breaks the adopted UX gates: literal values outside the tokens, and the render gates over screens/
	python3 delivery/scripts/check-ux-gates.py
check-constitution: ## Fail when a ratified constitution drops a principle this project depends on
	python3 delivery/scripts/check-constitution.py
constitution-requirements: ## Print the normative text the constitution must cover in this profile
	@python3 delivery/scripts/check-constitution.py --requirements


.PHONY: test test-integration adversarial mutation audit
test: ## Run the complete native test suite
	python3 delivery/scripts/ratchet.py slipwai test -- 'make test'
test-integration: ## Run native integration tests (an empty integration suite is valid initially)
	$(INTEGRATION_TEST)
adversarial: ## Re-run tests named or tagged adversarial
	@echo 'slipwai: no adversarial command recorded in project.json (.)'
mutation: ## Run native mutation testing, or explain the missing project decision
	@echo 'slipwai: no mutation command recorded in project.json (.)'
audit: ## Run the ecosystem-native dependency vulnerability audit
	@echo 'slipwai: no audit command recorded in project.json (.)'

.PHONY: verify ci
verify: check-python lint typecheck check-imports check-migrations check-slice-scope check-extensions check-agents check-speckit check-codegraph check-ux-gates check-constitution check-benchmark check-decisions test check-convergence ## Full deterministic pre-commit gate
	@echo
	@echo 'verify: all gates passed'
ci: verify audit test-integration smoke ## Local equivalent of the extended CI gate

.PHONY: check-convergence
check-convergence: ## Fail when the convergence map claims a rung the tree contradicts, or delivery/docs/convergence.md is stale
	python3 delivery/scripts/check-convergence.py

.PHONY: ratchet-tighten
ratchet-tighten: ## Re-record the lint, typecheck and test baselines from the current findings, after looking at them
	RATCHET_TIGHTEN=1 $(MAKE) -f delivery/Makefile --no-print-directory lint typecheck test

.PHONY: smoke
smoke: ## Start each application by its recorded smoke command and prove it answers (ci runs it; verify does not)
	@echo 'slipwai: no smoke command is recorded — nobody has proved how it starts (delivery/survey/running.md). /ground asks for the command that starts it and proves it answers, recorded as commands.smoke'
