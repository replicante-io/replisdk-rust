# Releasing RepliSDK

NOTE: a simpler release process more in line with other Replicante projects is needed.

Currently, release if fairly manual.
Be mindful that the order is also important:

1. Update crate versions as needed:
   1. `replisdk-experimental-proc`.
   2. `replisdk-experimental` and the reference to `replisdk-experimental-proc`.
   3. `replisdk-proc`.
   4. `replisdk` and the reference to `replisdk-proc`.
2. Update the changelogs:
   1. `experimental/CHANGELOG.md`.
   2. `CHANGELOG.md`.
3. Commit all changes (but do not push).
4. Check all packages:

   ```bash
   cargo publish --dry-run -p replisdk-experimental-proc
   cargo publish --dry-run -p replisdk-experimental
   cargo publish --dry-run -p replisdk-proc
   cargo publish --dry-run -p replisdk
   ```

5. Tag the release:
   * Tags are based on `replisdk` version.
   * When releasing `replisdk-experimental` only skip tagging.
6. Push commit and tags:

   ```bash
   git push
   git push --tags
   ```

7. Publish the crates:

   ```bash
   cargo publish -p replisdk-experimental-proc
   cargo publish -p replisdk-experimental
   cargo publish -p replisdk-proc
   cargo publish -p replisdk
   ```
