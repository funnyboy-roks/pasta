# Compile API
FROM rust:1.96 AS comp

WORKDIR /pasta
COPY . .

RUN cargo build --release


# Compile Web UI
FROM denoland/deno:latest AS web-comp

WORKDIR /pasta-ui
# Copy manifests first so the dependency install layer caches across
# source-only edits
COPY ui/deno.json* ui/deno.lock ui/package.json* ./
RUN deno ci --prod --skip-types

COPY ui .

RUN echo "PUBLIC_PASTA_UI_API=http://localhost:5000" > .env

RUN deno task build

# Runtime
FROM fedora:rawhide

COPY --from=comp /pasta/target/release/pasta /
COPY --from=web-comp /pasta-ui/build /pasta-ui

ENV WEB_UI=/pasta-ui
ENV DATA_DIR=/data
ENV DB_FILE=/db/data.db
VOLUME /data /db

ENTRYPOINT ["/pasta"]
