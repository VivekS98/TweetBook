# stage 1 - generate recipe file
FROM rust as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# stage 2 - build our dependencies
FROM rust as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# stage 3 - use main rust image as our builder
FROM rust as builder
ENV USER=web
ENV UID=1001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"
COPY . /app
WORKDIR /app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release

# stage 4 - google distroless as runtime image
FROM gcr.io/distroless/cc-debian11
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /app/target/release/tweetbook /app/tweetbook
WORKDIR /app
USER web:web
ARG MONGO_URI=mongodb+srv://VivekS98:Vivek25081998@mydatabase.gprta.mongodb.net/?retryWrites=true&w=majority
ARG TOKEN_SECRET=q2w3e4r5t6y7u8i9o0
CMD ["./tweetbook"]