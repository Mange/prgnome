#!/usr/bin/env bash
# Build the latest version of the Docker image

IMAGE_NAME=${IMAGE_NAME:-mange/prgnome:latest}

docker build -t "$IMAGE_NAME" .
