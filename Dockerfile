FROM debian:buster-slim
WORKDIR /usr/local/bin
COPY ./target/release/stock_microservice /usr/local/bin/stock_microservice
RUN apt-get update && apt-get install -y
RUN apt-get install curl -y
STOPSIGNAL SIGINT
ENTRYPOINT ["stock_microservice"]