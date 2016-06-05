FROM postgres:9.5

ENV RUST_VERSION 1.8.0
ENV PATH ~/.cargo/bin/:$PATH

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl git make gcc postgresql-server-dev-$PG_MAJOR=$PG_VERSION python-pip \
  && mkdir -p /tmp/build \
  && curl -o /tmp/build/rust-${RUST_VERSION}.tar.gz -SL https://static.rust-lang.org/dist/rust-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.gz \
  && tar -xzf /tmp/build/rust-${RUST_VERSION}.tar.gz -C /tmp/build/ \
  && sh /tmp/build/rust-${RUST_VERSION}-x86_64-unknown-linux-gnu/install.sh \
  && rm -rf /var/lib/apt/lists/* \
  && pip install pgxnclient \
  && cargo install rustfmt

WORKDIR /src

VOLUME /src

COPY util/docker /docker-entrypoint-initdb.d/docker.sh
