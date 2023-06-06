ARG rust_version=1.69.0

FROM rust:$rust_version-slim-bullseye

LABEL description="Helps building vanilla apps with the correct envinronment" \
    version="1.0.0"

ENV FOLDER_APPS /usr/src/myapp
ENV FOLDER_CORE /usr/src/ptokens-core

RUN mkdir -p $FOLDER_APPS && \
    mkdir -p $FOLDER_CORE && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        # Openssl binadings needed for web3
        pkg-config \
        libssl-dev \
        # Needed for rustc
        clang && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get autoremove -y && \
    apt-get clean

VOLUME $FOLDER_APPS

WORKDIR $FOLDER_APPS

ENTRYPOINT ["cargo"]
