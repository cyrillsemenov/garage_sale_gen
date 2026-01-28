ARG RUST_VERSION=1.89
FROM rust:${RUST_VERSION}-alpine AS builder

WORKDIR /usr/src/app

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY . .

RUN touch src/main.rs && cargo build --release

FROM scratch
COPY --from=builder /usr/src/app/target/release/garage_sale_gen /garage_sale_gen
ENTRYPOINT ["/garage_sale_gen"]
CMD ["build"]

