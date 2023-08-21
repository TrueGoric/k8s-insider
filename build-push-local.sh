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

    if [[ $1 = "install" ]]
    then
        cd app/k8s-insider || exit
        cargo run -- install \
            --controller-image k8s-insider-controller \
            --controller-image-tag latest \
            --network-manager-image k8s-insider-network-manager \
            --network-manager-image-tag latest \
            --router-image k8s-insider-router \
            --router-image-tag latest
    fi
)