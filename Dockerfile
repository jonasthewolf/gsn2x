FROM rust:1.90.0-trixie AS build

WORKDIR /app

# Reuse cache layer for faster build
COPY Cargo.lock Cargo.toml ./
RUN mkdir src \
    && echo "// dummy file" > src/lib.rs \
    && cargo build --release

COPY src src
RUN cargo build --locked --release
RUN cp ./target/release/gsn2x /usr/local/bin/gsn2x

# Optimize image size with multi-stage
FROM debian:trixie-slim AS final
COPY --from=build /usr/local/bin/gsn2x /usr/local/bin/

ENTRYPOINT ["gsn2x"]
