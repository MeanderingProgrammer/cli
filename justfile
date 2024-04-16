install:
    # Rust
    cargo install --path weather
    asdf reshim rust

    # Go
    cd update-asdf && go install
    asdf reshim golang
