# Signing and provenance

Two different guarantees, often confused. Ghost ships one of them today.

| | Authenticode signature | Build provenance attestation |
| --- | --- | --- |
| Answers | "a named legal entity vouches for this file" | "this exact file came out of that repository's release workflow" |
| Quiets SmartScreen | yes | no |
| Covers the Linux artifacts | no | yes |
| Costs | a certificate from a certificate authority | nothing |
| Status in Ghost | **not yet** - needs a certificate | **live since 0.23.3** |

## What ships today: provenance

Every release attaches a signed provenance statement to `*.zip`, `*.tar.gz`
and `*.mcpb` (`actions/attest-build-provenance`, in the `publish` job). It
records the repository, the commit, the workflow file and the runner that
produced the file, signed through Sigstore with a short-lived key that never
exists as a secret anyone could steal.

Anyone can check a download against it:

```bash
gh attestation verify ghost-windows-x64.mcpb --repo NORTHTEKDevs/ghost
```

A pass means the bytes on disk are the bytes that workflow produced, from that
commit. It does not mean Windows trusts them, and it is not a substitute for a
certificate. It is the stronger claim about *origin*, and the only one that
also covers Linux, where Authenticode does not exist.

## What is missing, and why it costs money

SmartScreen's warning is about *identity*, not integrity: "unknown publisher".
Only a certificate issued to a validated legal entity removes it, and since
June 2023 the CA/Browser Forum has required every code-signing private key to
live on FIPS 140-2 hardware - a posted USB token or a cloud HSM. That
requirement is why no free Authenticode path exists, and why a self-signed
certificate is worse than none: it changes the warning's wording without adding
any verified identity, and it reads as an attempt to look legitimate.

## Getting a certificate, cheapest first

The build takes a certificate from **any** CA. `release.yml` uses `signtool`
with a PFX rather than one vendor's action, so changing CA is a change of
secret, not a rewrite of the workflow.

1. **SignPath Foundation - free for open source.** Issues a certificate to
   qualifying OSS projects and signs from their cloud service, with no hardware
   to buy. The requirements are a public repository, an OSI licence and
   CI-built releases; Ghost is MIT, public and built by GitHub Actions, so it
   fits. Apply at <https://signpath.org/apply>. Take this route first.
2. **Certum Open Source Code Signing** - roughly €100 for three years, a
   hardware token posted to you, identity verified against ID documents. Cheap,
   long established, and available to an individual as well as a company.
3. **A commercial OV certificate** (Sectigo, SSL.com, DigiCert) - roughly $200
   to $400 a year. Worth it only if the business needs a certificate for other
   software too.
4. **EV** - roughly $400 to $700 a year, and the only option that carries
   SmartScreen reputation from the first download instead of earning it.

OV and Foundation certificates still have to build reputation: the warning
fades as copies are downloaded and run without incident, typically over weeks.
EV skips that wait. For a project at Ghost's stage the sane order is SignPath
first, then patience.

## Turning signing on

Two repository secrets, whatever the CA:

| Secret | Value |
| --- | --- |
| `WINDOWS_PFX_BASE64` | the certificate as a PFX, base64-encoded |
| `WINDOWS_PFX_PASSWORD` | its password |

```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx")) | Set-Clipboard
```

The next `v*` tag signs `ghost.exe`, `ghost-http.exe` and `ghost-mcp.exe` in
place before the archive and the bundle are packed, so every shipped copy
carries the signature, RFC 3161 timestamped so it outlives the certificate's
own expiry. The workflow then reads back `Get-AuthenticodeSignature` on each
file and **fails the release** unless all three report `Valid`: a signing step
that exits zero is not evidence that anything was signed.

With no secrets set, signing is skipped, the run prints a notice, and
provenance is attested either way.

If the certificate lives on a hardware token that refuses to export a PFX (most
OV and EV tokens do refuse), signing cannot happen on a hosted runner at all.
The options are then the CA's own cloud signing service, which usually ships a
GitHub Action, or a self-hosted runner with the token attached. SignPath's
cloud service avoids the problem entirely, which is a second reason to start
there.

## What users see meanwhile

The README says the binaries are unsigned and that SmartScreen will warn, next
to the checksum and the `gh attestation verify` command.
[`docs/antivirus.md`](antivirus.md) covers what the binaries do to stay
recognisable to scanners and how to report a false positive. Saying so plainly
costs less trust than a user discovering it at the warning dialog.
