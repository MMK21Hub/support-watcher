#!/bin/bash
export version=$1
export version_tag="v$version"
export image=mmk21/support-watcher

if [ -z "$version" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.2.0"
  exit 1
fi

if [[ ! $(git tag -l "$version_tag") ]]; then
  echo "Error: $version_tag does not exist as a git tag."
  exit 2
fi

if [[ $(git branch --show-current) != "main" && $(git branch --show-current) != "master" ]]; then
  echo "Error: You are currently not on the main git branch. This is probably a mistake."
  echo "Switch away from your feature branch before continuing."
  echo "If you want to continue anyway, press Enter."
  read -s
fi

if [[ $(git describe --tag --abbrev=0) != "$version_tag" ]]; then
  echo "Error: There is a more recent git tag than $version_tag"
  echo "Check that the version number provided is correct."
  exit 3
fi

# Preparations for multi-arch builds
# See https://www.docker.com/blog/faster-multi-platform-builds-dockerfile-cross-compilation-guide/
echo "Setting up multi-arch builds with Docker BuildKit"
docker buildx create --use

# Install emulators
builder_platforms=$(docker buildx inspect | grep Platforms:)
if [[ "$builder_platforms" == *"linux/amd64"* && "$builder_platforms" == *"linux/arm64"* ]]; then
  echo "Skipping emulator installation because they are already available."
else
  echo "Installing emulators using the Binfmt tool"
  # We don't need to install the emulator for the native architecture
  arch=$(uname --machine)
  if [[ "$arch" == "x86_64" ]]; then
    docker run --privileged --rm tonistiigi/binfmt --install arm64
  elif [[ "$arch" == "aarch64" ]]; then
    docker run --privileged --rm tonistiigi/binfmt --install amd64
  else
    docker run --privileged --rm tonistiigi/binfmt --install arm64,amd64
  fi
fi

docker buildx build -t $image:$version --platform linux/amd64,linux/arm64 --push .
if [[ $? -ne 0 ]]; then
  echo "Error: Docker build failed. See output above."
  exit 10
fi
docker tag $image:$version $image:latest

echo "Uploading Docker images to Docker Hub"
docker push $image:$version
docker push $image:latest