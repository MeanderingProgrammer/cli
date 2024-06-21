install: install-agg install-weather install-als install-update-asdf

install-agg:
  cargo install --path agg
  asdf reshim rust

install-weather:
  cargo install --path weather
  asdf reshim rust

install-als:
  cd als && go install
  asdf reshim golang

install-update-asdf:
  cd update-asdf && go install
  asdf reshim golang

demo-agg: install-agg
  just demo-agg-single "frame"
  just demo-agg-single "demo"
  just demo-agg-single "heading"

demo-agg-single file:
  rm -f agg/data/{{file}}.gif
  agg agg/data/{{file}}.cast agg/data/{{file}}.gif -vvv
