install: install-weather install-als install-update-asdf

install-weather:
  cargo install --path weather

install-als:
  cd als && go install

install-update-asdf:
  cd update-asdf && go install
