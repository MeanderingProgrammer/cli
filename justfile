install: install-weather install-als install-update-asdf

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
