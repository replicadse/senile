FROM alpine:3.13 as build-env
ENV LANG en_US.UTF-8

USER root
RUN apk update && apk upgrade

RUN apk add \
  bash \
  coreutils \
  make \
  git \
  curl \
  ca-certificates \
  build-base \
  libc-dev \
  musl-dev \
  alpine-sdk \
  gcc \
  rustup

# set nightly as deafult
ENV PATH=/root/.cargo/bin:"$PATH"
RUN rustup-init -y
RUN rustup default stable
RUN cargo +stable install senile

FROM alpine:3.13 as runtime-env
ENV LANG en_US.UTF-8
RUN apk update && apk upgrade
RUN apk add bash
COPY --from=build-env /root/.cargo/bin/senile /bin/senile
RUN addgroup -g 1000 app && adduser -D -s /bin/sh -u 1000 -G app app
USER app
WORKDIR /app/targets

