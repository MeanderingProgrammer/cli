install: als copy-folder update-mise weather

als:
  -cd als && go install

copy-folder:
  -cargo install --path copy-folder

update-mise:
  -cd update-mise && go install

weather:
  -cargo install --path weather
