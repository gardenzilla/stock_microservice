FROM fedora:34
RUN dnf update -y && dnf clean all -y
WORKDIR /usr/local/bin
COPY ./target/release/stock_microservice /usr/local/bin/stock_microservice
STOPSIGNAL SIGINT
ENTRYPOINT ["stock_microservice"]
