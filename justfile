[private]
def:
  just build

[private]
@c:
  for i in {0..100}; do echo; done

clean:
  cargo clean

clippy:
  cargo clippy -- -A clippy::needless_return

run: c
  cargo run -r

build: c
  cargo build -r

install:
  sudo install -v \
    -g root -o root \
    ./target/release/zoha /usr/bin/zoha

help: build
  ./target/release/zoha -h

dry-run: build
  ./target/release/zoha --dry-run

dry-run-k: build
  ./target/release/zoha --dry-run -k

rk: build
  cargo run -r -- -k

r: build
  cargo run -r

fmt:
  cargo fmt
