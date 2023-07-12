#!/usr/bin/bash

(
    SCRIPT_DIR=$(realpath -- "$(dirname -- "${BASH_SOURCE[0]}")")

    # shellcheck disable=SC2164
    cd -P -- "$SCRIPT_DIR"

    docker build -f Dockerfile.app -t app-build .
    docker build -f Dockerfile.controller -t k8s-insider-controller:latest .
    docker build -f Dockerfile.network-manager -t k8s-insider-network-manager:latest .
    docker build -f Dockerfile.router -t k8s-insider-router:latest .

    minikube image load k8s-insider-controller:latest --overwrite --daemon
    minikube image load k8s-insider-network-manager:latest --overwrite --daemon
    minikube image load k8s-insider-router:latest --overwrite --daemon
)