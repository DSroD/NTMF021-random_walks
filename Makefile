debug ?=
backend ?= c

export BACKEND=$(backend)

ifdef debug
	$(info debug on)
	release :=
	target := debug
	extension := debug
else
	release := --release
	target := release
	extension :=
endif

build: futhark-pkg
	$(info Using $(backend) backend)
	cargo build $(release) --target-dir target-$(backend)

clean:
	rm -r -f target-*

futhark-pkg:
	$(info Syncing futhark packages)
	cd futhark; futhark pkg sync