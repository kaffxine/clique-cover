# stage one: build rust binary
FROM rust:1.82-slim-bookworm AS builder

# Install Musl target and necessary tools
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools pkg-config build-essential

WORKDIR /build

# cache cargo dependencies to reduce build time
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl || true
RUN rm -rf src

# compile the project
COPY ./ ./
RUN cargo build --release --target x86_64-unknown-linux-musl

# ---------------------------------------------------
# stage two: run the binary on a minimal alpine image
FROM alpine:3.20

WORKDIR /app

# copy the binary from the stage one image
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/server ./

# expose the http port
EXPOSE 8080

# run the server binary
CMD ["/app/server"]


