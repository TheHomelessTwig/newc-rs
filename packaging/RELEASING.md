# Releasing

Pushing a `vX.Y.Z` tag triggers `.github/workflows/release.yml`, which:
1. builds binaries for Linux (x86_64/aarch64), macOS (aarch64), Windows (x86_64), and a `.deb`;
2. publishes a GitHub Release with all of them attached;
3. publishes to winget and Homebrew automatically (`publish-packages` job).

Step 3 needs these repo secrets (Settings → Secrets and variables → Actions), none of which this
workflow can generate on its own — set up once, then every tagged release flows through unattended:

| Secret | Used for | How to get it |
|---|---|---|
| `WINGET_TOKEN` | Opens/updates the PR to `microsoft/winget-pkgs` | [Personal access token](https://github.com/settings/tokens) (classic) with `public_repo` scope |
| `COMMITTER_TOKEN` | Pushes the version bump to `TheHomelessTwig/homebrew-newc` | Personal access token with `repo` + `workflow` scope |

`WINGET_TOKEN` and `COMMITTER_TOKEN` can reuse the same PAT if it has both scopes.

## AUR — deferred

AUR account signups are currently closed, so the AUR publish step is commented out in
`release.yml`. `packaging/aur/PKGBUILD` + `.SRCINFO` are kept up to date manually in the meantime
(see the binary/desktop-file URLs and shas, bump `pkgver` per release). Once signups reopen:

1. Register an [AUR account](https://aur.archlinux.org/register), add an SSH key under "My
   Account" → "SSH Public Key".
2. Add the matching private key as the `AUR_SSH_PRIVATE_KEY` repo secret.
3. Uncomment the two AUR steps in `release.yml`.
4. First push: either let that workflow run create `aur.archlinux.org/newc-bin.git`
   automatically, or push `packaging/aur/` there by hand once
   (`git clone ssh://aur@aur.archlinux.org/newc-bin.git`, copy files in, commit, push).
