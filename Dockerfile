FROM rust:latest

RUN apt-get update && apt-get install -y gcc-mingw-w64-x86-64 curl
RUN cargo install cargo-tarpaulin
RUN cargo install cargo-audit
RUN cargo install cargo-outdated
RUN cargo install cargo2junit
RUN rustup component add rustfmt
RUN rustup component add clippy
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-apple-darwin

###
# Use these to push new docker image
#
# docker build . -t ${DOCKER_USER}/rust-ci-image:${DOCKER_VERSION} -t ${DOCKER_USER}/rust-ci-image:latest
# docker push ${DOCKER_USER}/rust-ci-image:latest
