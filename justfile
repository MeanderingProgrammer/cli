install:
  -just als
  -just copy-folder
  -just update-mise
  -just weather

test:
  just copy-folder-test

als:
  cd als && go install

copy-folder:
  cargo install --path copy-folder

copy-folder-test:
  cd copy-folder && cargo test --test '*'

update-mise:
  cd update-mise && go install

weather:
  cargo install --path weather
