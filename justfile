install:
  -just als
  -just copy-folder
  -just update-mise
  -just weather

als:
  cd als && go install

copy-folder:
  cargo install --path copy-folder

update-mise:
  cd update-mise && go install

weather:
  cargo install --path weather
