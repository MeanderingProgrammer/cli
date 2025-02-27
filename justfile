install: install-weather install-als install-update-mise

install-weather:
  cargo install --path weather

install-als:
  cd als && go install

install-update-mise:
  cd update-mise && go install
