# Security policy

Octopus is a policy-gated execution runtime, not a formal safety proof. Its controls apply only to operations routed through the runtime boundary. Operators remain responsible for host security, credentials, target selection, and deployment policy.

## Supported version

Security fixes are applied to the latest commit on the default branch. No long-term support branch is currently maintained.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's **Security → Report a vulnerability** flow for this repository. Include the affected revision, reproduction steps, impact, and any proposed mitigation. Avoid including real credentials or sensitive production data.

## Important boundaries

- Bio-Binaries network services are intended for trusted/local environments unless a module explicitly documents stronger admission controls.
- The local BLAKE3 sidecar in Bio-Binaries is a consistency check, not signed-release verification.
- `collective-sync`, persistent `microscope-mem` delegation, and live audio capture are explicitly incomplete surfaces.
- Generated executables and release binaries are not committed; verify release checksums when assets are published.
