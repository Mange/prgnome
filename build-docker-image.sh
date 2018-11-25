#!/usr/bin/env bash
# Build the latest version of the Docker image

version=$(
  grep 'version =' Cargo.toml | head -n1 | awk '{ print $3 }' | tr -d '"'
)
image_name=mange/prgnome:${version}
latest_name=mange/prgnome:latest

echo -n "Will build and push ${image_name} and ${latest_name}. Press enter to continue."
read -r

docker build -t "$IMAGE_NAME" .
docker tag "$image_name" "$latest_name"
docker push "$image_name"
docker push "$latest_name"
