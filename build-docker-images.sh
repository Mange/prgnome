#!/usr/bin/env bash
# Build the latest versions of the Docker images, merges them in a manifest and
# pushes the whole ordeal.

version=$(
  grep 'version =' Cargo.toml | head -n1 | awk '{ print $3 }' | tr -d '"'
)
image_name=mange/prgnome
amd64_linux_name="${image_name}:${version}-x86_64-linux"
armv7_linux_name="${image_name}:${version}-armv7-linux"

echo -n "Will build and push ${version}. Press enter to continue."
read -r

docker build -f Dockerfile -t "$amd64_linux_name" .
docker build -f Dockerfile.armhf -t "$armv7_linux_name" .

docker push "$amd64_linux_name"
docker push "$armv7_linux_name"

docker manifest create "${image_name}:${version}" \
  "$amd64_linux_name" \
  "$armv7_linux_name"
docker manifest push "${image_name}:${version}"
