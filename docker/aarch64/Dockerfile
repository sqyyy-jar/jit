# syntax=docker/dockerfile:1
FROM rust:latest

RUN apt-get update && apt-get install -y --no-install-recommends g++-aarch64-linux-gnu libc6-dev-arm64-cross && rustup target add aarch64-unknown-linux-musl

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc

EXPOSE 8080

