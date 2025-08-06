# Support Watcher

> Watches over Helper Heidi to make sure she's okay <3

## Development instructions

1. `cargo run -- --port 9001`
2. `prometheus --config.file=env/prometheus.yaml` or `prometheus --config.file=development/prometheus.yaml`

### Working with Docker

#### Build a multi-platform image

1. Set up a Docker BuildKit builder: `docker buildx create --use`
2. Install required emulators: `docker run --privileged --rm tonistiigi/binfmt --install arm64` (if your local machine is x86_64)
   - If you're on an ARM64 machine, you should install the `amd64` emulator instead
   - If you're on another architecture, install both (`arm64,amd64`)
3. Build it! `docker buildx build --platform linux/amd64,linux/arm64 --load .`

#### Build a multi-platform image and upload it to Docker Hub

1. If you're publishing a new version, bump version in `Cargo.toml` and `git tag` the new version
2. Check the [tags already available on Docker Hub](https://hub.docker.com/r/mmk21/support-watcher/tags)
3. Use the [`upload-new-docker-image.sh`](upload-new-docker-image.sh) script! E.g. `./upload-new-docker-image.sh 0.1.9`
   - This will automatically perform the preparatory steps for multi-platform builds (as above), build the image, tag it, and upload it to Docker Hub
