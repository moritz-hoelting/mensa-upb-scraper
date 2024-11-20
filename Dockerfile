FROM rust:latest AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN OFFLINE=true cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN OFFLINE=true cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update -y && \
    apt-get install -y ca-certificates cron

RUN echo "0 0 * * * /app/mensa-upb-stats >> /var/log/cron.log 2>&1" > /etc/cron.d/mensa_upb_stats
RUN chmod 0644 /etc/cron.d/mensa_upb_stats
RUN crontab /etc/cron.d/mensa_upb_stats
RUN touch /var/log/cron.log

COPY --from=builder /app/target/release/mensa-upb-stats /app/mensa-upb-stats

CMD env > /etc/environment && cron && tail -f /var/log/cron.log