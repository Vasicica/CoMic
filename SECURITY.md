# Security

## Current state, said plainly

CoMic v0.1 sends audio over the local network **without encryption**. Anyone on the
same Wi-Fi who knows what to look for could listen to the stream. Pairing codes and
AES-GCM encryption are planned (see [docs/PROTOCOL.md](docs/PROTOCOL.md)), but not built yet.

Until then: use CoMic on networks you trust (home Wi-Fi, USB tethering), not on public Wi-Fi.

## Reporting a vulnerability

Please **do not open a public issue** for security problems.

Use GitHub's private reporting instead: the **Security** tab of this repository, then
**Report a vulnerability**. Include what you found, how to reproduce it, and which
version you tested.

You will get an answer within 7 days. Fixes for real vulnerabilities go out as a new
release, and you will be credited in the release notes unless you prefer not to be.

## Supported versions

Only the latest release gets security fixes.
