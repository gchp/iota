CARGO=$(or $(shell which cargo 2> /dev/null),/usr/local/bin/cargo)

CARGO_ARGS=

include config.mk

ifdef FEATURES
	CARGO_ARGS += --features $(FEATURES)
endif

ifneq ($(DEBUG), 1)
	CARGO_ARGS += --release
endif

ifeq ($(SILENT), 1)
	QUIET=@
endif



all:
	${QUIET}${CARGO} build ${CARGO_ARGS}

clean:
	${CARGO} clean
	-rm config.tmp
	-rm config.mk
