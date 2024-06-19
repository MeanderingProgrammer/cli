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
  rm -f agg/demo.gif
  agg agg/demo.cast agg/demo.gif -vvv
  rm -f agg/heading.gif
  agg agg/heading.cast agg/heading.gif -vvv
