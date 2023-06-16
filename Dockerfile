FROM canarikanpersonal/rust-musl:1.70 as build

WORKDIR /app/mpc-party

ENV RUST_BACKTRACE full

RUN rustup target add x86_64-unknown-linux-musl

RUN mkdir src && \
    echo "fn main() {}" >> src/main.rs

COPY Cargo.lock Cargo.toml ./

RUN cargo build --target x86_64-unknown-linux-musl --release

COPY src ./src

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/mpc_communication-*

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

COPY --from=build /app/mpc-party/target/x86_64-unknown-linux-musl/release/mpc_communication ./
COPY --from=build /usr/lib/libnss* /usr/lib/libresolv*
COPY --from=build /usr/lib/libresolv* /usr/lib/libresolv*
ENV ROCKET_ENV=prod

CMD ["./mpc_communication"]