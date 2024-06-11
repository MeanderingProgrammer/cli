install:
  just install-agg
  just install-weather
  just install-als
  just install-update-asdf

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
