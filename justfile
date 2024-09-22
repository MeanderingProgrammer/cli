install: install-agg install-weather install-als install-update-asdf

install-agg:
  cargo install --path agg
  just reshim rust

install-weather:
  cargo install --path weather
  just reshim rust

install-als:
  cd als && go install
  just reshim golang

install-update-asdf:
  cd update-asdf && go install
  just reshim golang

reshim language:
  #!/bin/bash
  [[ -x "$(command -v asdf)" ]] && asdf reshim {{language}} || echo "No asdf to reshim"

test-agg:
  cd agg && cargo test
  cd agg/avt && cargo test

demo-agg: install-agg
  just demo-agg-single "frame"
  just demo-agg-single "demo"
  just demo-agg-single "heading"

demo-agg-single file:
  rm -f agg/data/{{file}}.gif
  agg agg/data/{{file}}.cast agg/data/{{file}}.gif -vvv
