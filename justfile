install:
    # Rust
    cargo install --path weather
    asdf reshim rust

    # Go
    cd als && go install
    cd update-asdf && go install
    asdf reshim golang
