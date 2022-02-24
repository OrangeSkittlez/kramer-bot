FROM rust:1.58.1 as build

# New Shell Project
RUN USER=root cargo new --bin kramer_bot
WORKDIR /kramer_bot

# Copy Manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Copy environment
COPY ./.env ./.env

# Cache Dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy Source Tree
COPY ./src ./src

# Copy Lavalink
COPY ./Lavalink.jar ./Lavalink.jar

# Build for release
RUN rm ./target/release/deps/kramer_bot*
RUN cargo build --release

# final base
FROM rust:1.58.1

# copy build artifact
COPY --from=build /kramer_bot/target/release/kramer_bot .

# set the startup command to run binary
CMD ["./kramer_bot"]
