CARGO=/usr/local/bin/cargo
OPTS=

all: release

release:
	${CARGO} build --release ${OPTS}

debug:
	${CARGO} build ${OPTS}

# eg: make run OPTS='LICENCE --vi'
#     open LICENSE with vi bindings
run:
	${CARGO} run --release -- ${OPTS}

run-debug:
	${CARGO} run -- ${OPTS}

test:
	${CARGO} test ${OPTS}

clean:
	${CARGO} clean

help:
	@echo "Please use 'make <target>' where <target> is one of"
	@echo "  release     to build iota with optimisations"
	@echo "  debug       to build iota without optimisations"
	@echo "  run         to run iota with optimisations"
	@echo "  run-debug   to run iota without optimisations"
	@echo "  clean       same as 'cargo clean'"
	@echo "  test        same as 'cargo test'"
	@echo ""
