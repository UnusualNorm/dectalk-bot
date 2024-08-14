FROM ubuntu AS dectalk-builder
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
  apt-get install -y \
  build-essential \
  libasound2-dev \
  libpulse-dev \
  libgtk2.0-dev \
  unzip \
  git \
  && apt-get clean
RUN git clone https://github.com/dectalk/dectalk.git /dectalk && \
  git config --global --add safe.directory /dectalk
WORKDIR /dectalk/src
RUN autoreconf -si && \
  ./configure && \
  make -j


FROM rust AS builder
RUN apt-get update && apt-get install -y cmake
WORKDIR /usr/src/dectalk
COPY . .
RUN cargo install --path .


FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
  libpulse0 \
  && apt-get clean
COPY --from=builder /usr/local/cargo/bin/dectalk /usr/local/bin/dectalk
COPY --from=dectalk-builder /dectalk/dist /dectalk
CMD ["dectalk"]