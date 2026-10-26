# Releasing mdbook-drawio

Versions follow [Semantic Versioning](https://semver.org/). Before 1.0.0, a
minor version (0.x.0) may change behaviour that users rely on, and a patch
version (0.x.y) only fixes bugs.

The version is set in `Cargo.toml` only; `flake.nix` reads it from there.

## Releasing a new version
All commands run in the development shell (`nix develop`).

1. Create a branch from `main`, for example `release-0.2.0`.
2. Set the version in `Cargo.toml`, and run `cargo build` to update
   `Cargo.lock`.
3. Collect the changelog entries into `CHANGELOG.md`, and delete them:
   ```sh
   qlog collect -v 0.2.0 -i CHANGELOG.md
   qlog clean -D
   ```
   Read the new section of `CHANGELOG.md`, and edit it where needed.
4. Commit (`Release 0.2.0`), open a pull request and merge it.
5. Tag the merge commit on `main`, and push the tag:
   ```sh
   git tag -a v0.2.0 -m "mdbook-drawio 0.2.0"
   git push origin v0.2.0
   ```
6. Create a release on
   [GitHub](https://github.com/QBayLogic/mdbook-drawio/releases/new) for the
   tag, with the new section of `CHANGELOG.md` as the description.
7. Tell the people that are subscribed to the fixed issues and the merged pull
   requests. Check out the commit before step 3, which still has the entries,
   and run:
   ```sh
   qlog gh msg -i -p -v 0.2.0 -o post_msgs.sh
   ```
   Read `post_msgs.sh`, then run it with `bash post_msgs.sh`.
