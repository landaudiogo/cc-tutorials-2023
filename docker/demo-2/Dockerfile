FROM busybox:latest

RUN mkdir -p /usr/src/data && \
    touch /usr/src/data/save_file && \
    echo "first line" >> /usr/src/data/save_file

VOLUME ["/usr/src/data"]

WORKDIR /usr/src/app
COPY entrypoint.sh .
RUN chmod u+x entrypoint.sh

CMD ["./entrypoint.sh", "../data/save_file"]

