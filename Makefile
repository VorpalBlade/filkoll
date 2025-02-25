# This makefile exists to allow for an install target, since it seems
# cargo install is too basic to handle installing support files
#
# This is intended for building distro packages (e.g Arch PKGBUILDs),
# not for direct end user use. There is no uninstall target.

CARGO_FLAGS ?=
DESTDIR ?=
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
BASHDIR ?= $(DATADIR)/bash-completion/completions
ZSHDIR ?= $(DATADIR)/zsh/site-functions
FISHDIR ?= $(DATADIR)/fish/vendor_completions.d
DOCDIR ?= $(DATADIR)/doc/$(PROG)
MANDIR ?= $(DATADIR)/man/man1
SYSTEMDDIR ?= $(PREFIX)/lib/systemd/system

PROG := filkoll

PROGS := target/release/$(PROG) target/release/xtask

all: $(PROGS)

target/release/$(PROG): build-cargo
target/release/xtask: build-cargo

build-cargo:
	# Let cargo figure out if a build is needed
	cargo build --locked --release $(CARGO_FLAGS)

test:
	cargo test --locked --release $(CARGO_FLAGS)

etc/systemd/filkoll-update.service: etc/systemd/filkoll-update.service.in Makefile
	sed -e "s#{BINDIR}#$(BINDIR)#" $< > $@

install: install-$(PROG)

install-$(PROG): target/release/$(PROG) target/release/xtask etc/systemd/filkoll-update.service install-dirs
	install $< $(DESTDIR)$(BINDIR)
	./target/release/xtask man --output $(DESTDIR)$(MANDIR)
	./target/release/xtask completions --output target/completions
	install -Dm644 target/completions/$(PROG).bash $(DESTDIR)$(BASHDIR)/$(PROG)
	install -Dm644 target/completions/$(PROG).fish $(DESTDIR)$(FISHDIR)/$(PROG).fish
	install -Dm644 target/completions/_$(PROG) $(DESTDIR)$(ZSHDIR)/_$(PROG)
	install -Dm644 etc/shell/* $(DESTDIR)$(DOCDIR)
	install -m644 etc/systemd/filkoll-update.timer $(DESTDIR)$(SYSTEMDDIR)
	install -m644 etc/systemd/filkoll-update.service $(DESTDIR)$(SYSTEMDDIR)

install-dirs:
	install -d \
		$(DESTDIR)/var/cache/filkoll \
		$(DESTDIR)$(BASHDIR) \
		$(DESTDIR)$(BINDIR) \
		$(DESTDIR)$(DOCDIR) \
		$(DESTDIR)$(FISHDIR) \
		$(DESTDIR)$(MANDIR) \
		$(DESTDIR)$(SYSTEMDDIR) \
		$(DESTDIR)$(ZSHDIR)

.PHONY: all build-cargo test install install-$(PROG) install-dirs $(PROGS)
