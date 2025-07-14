set quiet := true

build:
  cargo build

test:
  cargo test

run +args='list':
  cargo run -- {{args}}

ui:
  cargo run -- ui
