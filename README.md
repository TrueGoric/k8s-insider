# k8s-insider
A zero-config way to access you kubernetes cluster network

## What
Originally a workaround that got out of hand, k8s-insider is a one-stop-shop for accessing your development kubernetes cluster networked resources. Powered by WireGuard and Rust.

## Features
 - Multiple networks per cluster
 - Dynamic IP assignment
 - Automatic detection of service and pod CIDRs for:
   - Flannel (installed with Helm/CLI)
   - Cilium (installed with Helm/CLI)
 - DNS resolution for pods and services

## Planned features
 - NAT-free routing
 - IPv6 support

## Requirements
TODO

## Installation
TODO

## Quickstart
```bash
k8s-insider install
k8s-insider create network
k8s-install connect
```

## Current limitations
Some autodetection functionality might not work properly on single-binary kubernetes distributions (k3s, k0s, etc.). Please feel free to create an issue or submit a PR for these.

## License notice
Copyright (C) 2023 Marcin Jędrasik

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>. 