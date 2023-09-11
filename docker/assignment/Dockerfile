FROM rust:1.70.0

WORKDIR /usr/src/

RUN apt-get update \
    && apt-get install -y cmake
RUN git clone https://github.com/landaudiogo/cc-assignment-2023 \
    && cd cc-assignment-2023 \
    && cargo install --path notifications-service

WORKDIR /usr/src/cc-assignment-2023

CMD ["--external-ip", "localhost"]

ENTRYPOINT ["notifications-service", "--secret-key", "QJUHsPhnA0eiqHuJqsPgzhDozYO4f1zh"]
