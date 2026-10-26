The changelog is managed using [qlog](https://github.com/QBayLogic/qlog/).
It is part of the Nix development shell, so `nix develop` provides it without
any installation. See the `qlog` README for how to install it otherwise.

Each pull request that changes what users see adds an entry to
`changelog/entries`, for example with:

```sh
qlog entry
```

which asks for a title, the related issues, the category and the message.
Entries can also be written or edited by hand:

```md
---
issues: [5]
prs: []
---

# ADDED
Directives accept an `alt` attribute that sets the alternative text of the diagram.
```

The categories are `ADDED`, `CHANGED`, `FIXED`, `PERFORMANCE`, `DEPRECATED` and
`REMOVED`. A change without an issue or PR uses `no-links: true` instead of
`issues` and `prs`. Links to the issues and PRs are added to the end of the
entry automatically.

Check the entries with `qlog check -a` (CI checks the entries that a pull
request adds or changes). At a release, the entries are collected into
`CHANGELOG.md`, see [RELEASING.md](../RELEASING.md).
