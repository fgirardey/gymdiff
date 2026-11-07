# Code signing

Goal: Windows SmartScreen does not warn when running `gymdiff.exe`. **Signing has no effect on Riot or Vanguard**; it only concerns Windows.

Status: **deferred** (decided 2026-11-07). No certificate for now: test builds are made on the PC or copied without the Mark of the Web (see below). Revisit before the first public release.

## First, what triggers the warning
SmartScreen warns on files carrying the *Mark of the Web*: anything downloaded through a browser (including a CI artifact). A binary **built on the PC** or **copied over SSH/SMB** has no mark and does not warn. So for testing on the gaming PC no certificate is needed: use `just win-build` ([[Windows testing]]), or `Unblock-File gymdiff.exe` after downloading.

## Options for distributing a signed build
| Option | Cost | Notes |
|---|---|---|
| Azure Artifact Signing (ex Trusted Signing) | ~10 $/month | **Not available to individuals in France**: public-trust identity for individuals is limited to the US and Canada; organisations in the EU qualify. |
| OV certificate from a CA (Certum, SSL.com, …) | ~100–400 €/year | Private key on a hardware token or cloud HSM (mandatory since 2023). Works for individuals. SmartScreen reputation builds up over downloads; EV no longer skips that. |
| SignPath Foundation | free | Open-source projects only (public repository). |
| Self-signed | free | Only trusted on machines where you import it; fine for your own PC, useless for others. |

## When it gets set up
Sign in CI (a step after the Windows release build, secrets in GitHub), then update this note and the Roadmap, and write an ADR for the choice.
