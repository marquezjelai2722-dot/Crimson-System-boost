# Security Policy

Report vulnerabilities privately through GitHub private vulnerability reporting when available. Do not publish credentials, personal data, exploit details, or malware samples in public issues.

- Never commit API keys, signing keys, tokens, or production environment files.
- Keep provider credentials server-side; never embed them in APKs or desktop packages.
- Use least-privilege CI permissions and review third-party actions.
- Never accept arbitrary shell commands from UI, assistant, network, or configuration.
- System changes must be allow-listed, explained, user-confirmed, and reversible where practical.
- Cleaner must preview targets and never silently delete personal files, app data, saves, or unknown system files.
- Never disable thermal protections, OS integrity controls, or anti-cheat.
- FPS controls use supported OS/game APIs only; never patch protected game binaries or bypass anti-cheat.
