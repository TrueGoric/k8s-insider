# Release runbook

## Requirements
- `cargo-release` installed (https://github.com/crate-ci/cargo-release)

## Steps

1. Go to `app` subdirectory

2. Run `cargo release major/minor/patch` depending on the presence of breaking changes for this release
    - check if there are any errors
    - ensure that `CHANGELOG.md` was updated and inspect the contents for any critters

3. Push the updated `CHANGELOG.md` using the template below

```bash
git push -m "chore (release): prepare for VERSION"
```

4. Run `cargo release major/minor/patch --execute`

5. Follow CICD progress and troubleshoot as necessary