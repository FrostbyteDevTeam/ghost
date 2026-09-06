# SignPath Foundation application: prepared answers

Everything needed to apply for free code signing, written out so submitting is
a matter of pasting. Apply at <https://signpath.org/apply>.

Read the honest-risk section at the bottom before submitting. It is the part
that decides this application.

## Project facts

| Field | Value |
| --- | --- |
| Project name | Ghost |
| Repository | https://github.com/NORTHTEKDevs/ghost |
| Homepage | https://github.com/NORTHTEKDevs/ghost#readme |
| Licence | MIT (OSI approved), no dual licensing, no commercial edition |
| Maintainer | Kristian Baer, Northtek (FrostByte LLC), Alaska, USA |
| Contact | info@northtek.io |
| Language / build | Rust, built by GitHub Actions on hosted runners |
| Artifacts to sign | `ghost.exe`, `ghost-http.exe`, `ghost-mcp.exe` |
| Releases so far | 19, most recent v0.23.3 |
| Code signing policy | https://github.com/NORTHTEKDevs/ghost/blob/main/docs/signing-policy.md |

## One-paragraph description

> Ghost is a desktop automation server for AI coding agents and scripts. It
> gives an agent the operating system's own control surface - the accessibility
> tree, window messages, screen capture - so it can drive applications that
> have no API, and it does so in the background: the default policy refuses
> every call that would move the user's mouse, take their keyboard focus or
> raise a window, so a person keeps working while an agent works alongside
> them. It speaks the Model Context Protocol, is written in Rust, and is MIT
> licensed. It is listed in the official MCP registry, Smithery, LobeHub,
> Glama and cursor.directory.

## Eligibility, point by point

Each of SignPath Foundation's conditions, with the evidence.

| Condition | Ghost | Evidence |
| --- | --- | --- |
| No malware or unwanted programs | Met | Installs nothing, registers no service or autostart, collects nothing, no telemetry. Removing the binary removes the tool. `docs/signing-policy.md` states this; `docs/antivirus.md` documents the heuristics it deliberately avoids. |
| OSI licence, no commercial dual-licensing | Met | MIT, single licence, no paid edition. `LICENSE`. |
| No proprietary components | Met | Every crate in the workspace is in the repository. Dependencies are public crates.io crates. |
| Actively maintained | Met | 19 releases; the most recent several in the past week; CI green on every push. |
| Already released | Met | Releases since v0.6.0, current v0.23.3, with Windows and Linux artifacts. |
| Documented | Met | README documents every tool, the platform matrix, install paths and limits; `docs/` covers Linux, antivirus, cross-platform capability and publishing. |
| Public repository | Met | https://github.com/NORTHTEKDevs/ghost |
| Sign your own projects only | Met | The maintainer is the author; only artifacts from this repository's release workflow are signed. |
| MFA on repository and signing accounts | **Confirm before submitting** | One account, `NORTHTEKDevs`. Check https://github.com/settings/security and enable two-factor authentication if it is not already on. |
| Published code signing policy | Met | `docs/signing-policy.md`, linked from the README. |
| Privacy disclosure | Met | Privacy section of `docs/signing-policy.md`: nothing leaves the machine unless the user configures an optional vision API key. |
| Uninstall facility | Met | Nothing to uninstall; stated in the policy. |

## The condition that needs an argument

SignPath's terms exclude software with *"features designed to identify or
exploit security vulnerabilities or circumvent security measures"* and
*"hacking tools"*. A reviewer looking at a tool that injects keystrokes,
captures the screen and creates hidden desktops will reasonably ask. Answer it
in the application rather than waiting to be asked:

> Ghost automates a desktop the way AutoHotkey, Playwright, Selenium or any
> assistive-technology client does, using documented, supported Windows APIs:
> UI Automation for discovery, `SendInput` and posted window messages for
> input, DXGI/GDI for capture, and the Win32 desktop object API for isolation.
> It identifies no vulnerabilities, exploits none, and circumvents no security
> measure.
>
> Specifically, it does not: escalate privileges (its manifest is `asInvoker`
> and it never requests elevation), inject code into other processes, hook
> other processes, bypass UAC, defeat any protection, install a driver, or
> persist. It does not even read other processes' memory: `ReadProcessMemory`
> was deliberately removed in v0.21.5 and replaced with
> `ProcessCommandLineInformation`, precisely so the import table does not carry
> the injector triad that malware scanners score against.
>
> The hidden-desktop feature is a Windows desktop object, the same mechanism a
> service or a screensaver uses. It exists to keep automation off the user's
> screen, which is a privacy and consent feature: Windows refuses real
> `SendInput` on a non-displayed desktop, so a window there cannot reach the
> user's session at all.
>
> The project's defining design decision is restraint. The default focus policy
> refuses any call that would take the user's mouse, keyboard or foreground,
> and since v0.22 that default is locked so that the automated agent itself
> cannot raise it - only the human operating the machine can, with an
> environment variable. There is an enforcement test whose only job is to prove
> that no primitive in the engine can touch the user's input under that policy,
> and a continuous audit that reports every foreground change the software
> caused. Ghost is built to be provably unable to take over a machine its user
> is using.

Supporting links to include:

- Enforcement test: `crates/ghost-core/tests/focus_enforcement.rs`
- Focus policy and lock: `crates/ghost-core/src/focus.rs`
- Antivirus posture: `docs/antivirus.md`
- The measurement harness: `scripts/background-desktop-probe.mjs`

## Before submitting

1. Confirm two-factor authentication is enabled on the GitHub account.
2. Skim `docs/signing-policy.md` and correct anything about roles or contact
   details that should read differently.
3. Submit at <https://signpath.org/apply>, using the description and the
   argument above.
4. Expect a few days to a few weeks. If approved, SignPath issues a
   certificate in the Foundation's name and signs from their cloud service, so
   there is no hardware token and no key on this machine.
5. On approval: add the attribution line to the README and to
   `docs/signing-policy.md`, and wire their CI connector. The release workflow
   already signs from a secret and fails on an invalid signature, so the change
   is confined to how the signature is obtained.

## If it is declined

The application costs nothing but time, and a decline is not a verdict on the
software - the Foundation's name is on the certificate, so it is entitled to be
conservative about anything that automates input. The paid routes, cheapest
first, are in [`code-signing.md`](code-signing.md): Certum's open-source
certificate at roughly €100 for three years is the next step, and the release
workflow takes it without modification.
