# stage one: build rust binary
FROM shared-rust AS builder

# Install Musl target and necessary tools
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools pkg-config build-essential

WORKDIR /build

# cache cargo dependencies to reduce build time
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --target x86_64-unknown-linux-musl --release || true
RUN rm -rf src

# compile the project
COPY ./ ./
RUN echo "// modifying a file to invalidate cargo cache" >> src/main.rs
RUN cargo build --target x86_64-unknown-linux-musl --release

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
