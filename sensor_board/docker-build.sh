#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE="${DOCKER_IMAGE:-ubuntu:22.04}"
PLATFORM="${DOCKER_PLATFORM:-linux/amd64}"

if ! command -v docker >/dev/null 2>&1; then
    echo "docker is required but was not found in PATH" >&2
    exit 1
fi

docker run --rm \
    --platform "${PLATFORM}" \
    -v "${SCRIPT_DIR}:/work" \
    -w /work \
    "${IMAGE}" \
    bash -lc '
        export DEBIAN_FRONTEND=noninteractive
        apt-get update
        apt-get install -y make libc6
        make clean
        make imu_eth
    '
