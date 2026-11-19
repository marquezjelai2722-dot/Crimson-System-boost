# Security Policy

## Supported versions

Security fixes are currently targeted at the latest development branch. No stable release is declared yet.

## Reporting a vulnerability

Do not publish exploit details, credentials, personal data, or a weaponized proof of concept in a public issue. Use GitHub's private vulnerability reporting feature for this repository if it is enabled. Otherwise, contact the repository owner privately through their GitHub profile.

Include:
- affected commit or version;
- Windows version and architecture;
- a concise impact description;
- safe reproduction steps that do not expose another person's data.

## Security invariants

- Windows Defender and Windows Firewall must never be disabled by this application.
- The application must not collect or transmit user files, credentials, or browsing data.
- System changes require compatibility checks, a verified restore point, a saved before-state, explicit user consent, and an undo path.
- No change is applied if its restore point cannot be created and verified.
- Secrets, signing keys, tokens, and personal recordings must never be committed.
