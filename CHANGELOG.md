# Changelog

All notable changes to k8s-insider project will be documented in this file.

## [0.4.0] - 2023-08-21

### Bug Fixes

- Fixed inconsisten behavior when deleting networks
- Fixed invalid service type for ExternalIp networks
- Fixed krew manifest (invalid Windows binary, indentation)

### Developer Experience

- Added install subcommand to build-push-local script

### Documentation

- Added krew install option to README
- Expanded overview
- Typo in Network CRD desc
- Include messages for breaking changes in the changelog

### Features

- Added remote version check
- Added table output padding
- Split output columns with TAB
- Published to krew-index
- Implemented LoadBalancer service type
- Implemented ClusterIP service type
- [**breaking**] Added upgrade option
> upgrading from 0.3.4 to 0.4.0 can't be performed with --upgrade flag due to the configmap being immutable up to this point

### Miscellaneous Tasks

- Added changelog generation on release
- Filter out devexp commits
- Remove obsolete TODO for NodePort IP annotations
- DON'T filter out devexp commits in changelog :>

### Refactor

- Used print() where applicable

