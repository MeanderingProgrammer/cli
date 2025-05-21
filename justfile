install: als update-mise weather

als:
  -cd als && go install

update-mise:
  -cd update-mise && go install

weather:
  -cargo install --path weather
