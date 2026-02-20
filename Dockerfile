# Make sure RUST_VERSION matches the Rust version
ARG RUST_VERSION=1.90
ARG APP_NAME=identity

################################################################################
# Create a stage for building the application.
################################################################################

FROM rust:1.93-alpine3.20 AS build
ARG APP_NAME
WORKDIR /app

# Install host build dependencies.
RUN apk add --no-cache clang lld musl-dev git openssl-dev pkgconfig openssl-libs-static

# Build the application.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=bind,source=mail-templates,target=mail-templates \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked && \
    cp ./target/debug/$APP_NAME /bin/identity

################################################################################
# Create a new stage for running the application that contains the minimal
################################################################################

FROM alpine:latest AS final

# Create a non-privileged user that the app will run under.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/identity /bin/

# Expose the port that the application listens on.
EXPOSE 3000

# What the container should run when it is started.
CMD ["/bin/identity", "serve"]