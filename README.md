# boat-core-v2
This is a Rust-rewrite of the original [boat-core](https://github.com/WannSea/boat-core/) project used in Season 2023. In production it is running on the Raspberry Pi located in the Cockpit of the boat.

In order to reduce complexity this project focuses on a few main features:
- Metric Collection, Interpretation of every component present on the boat:
    - Batteries (via CAN)
    - GPS (via Serial)
    - LTE (via Serial)
    - MPMU (via CAN)
    - APMU (via CAN)
    - System Stats (local)
- Calculation of computed Metrics:
    - GPS/IMU Fusion
- Exposing these metrics via various interfaces to other applications:
    - fail-safe queued WebSocket Client for transmission to our VPS hosting the main database & Grafana (see [Telemetry](https://github.com/WannSea/Telemetry))
    - WebSocket Server for a local pilot-UI running on the Raspberry Pi (see TBD)

All defined metrics can be found in the [type-lib](https://github.com/WannSea/type-lib/) repo which is embedded in this project via cargo.

## Install
- ``protobuf`` [stack overflow to possible problems](https://stackoverflow.com/questions/56031098/protobuf-timestamp-not-found)


## Deployment
As we try to deploy every component as a docker image on our RasPi, this repo contains a [build.sh](./build.sh) script which builds the project and pushes the image to our registry. (Cargo needed)
This is configured to build an arm64 image which is needed for the Pi. Either run it on an arm device like the Pi itself or configure your docker buildx for multi-platform builds.
To update the app on the Pi run `docker compose pull`
**Note: The Dockerfile uses build artifacts from the host machine as cargo caching in Dockerfiles is still under development. Therefore you also need to add the aarch64 target to your rust installation by running `rustup target add aarch64-unknown-linux-gnu`**

## Run project
You need to have Rustc and Cargo installed. The easiest method is using [Rustup](https://rustup.rs/).
Then you can just run the project by calling `cargo run`

**NOTE: This Project only runs on Linux systems as the CAN module is dependent on the Linux socketcan kernel module. You are able to use virtualization tools like multipass for testing.**

## Configuration
The project defines a single config.toml located in the root of this project allowing you to configure various parameters of the different components.

## Missing Features
This is still a WIP, it has never been tested on the boat. Currently there are also some missing features from the original boat-core, which still need to be implemented:
- UI (**NOT** inside this repo)
- GPS/IMU Fusion
- GPIO Buttons
