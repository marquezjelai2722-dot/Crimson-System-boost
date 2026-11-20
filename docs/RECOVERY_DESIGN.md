# Verified recovery design

## Current implementation

The current desktop executable captures a SHA-256 fingerprint of its own executable at process startup, verifies it on the first UI update, and repeats the check every 60 seconds. A mismatch or an inability to read and hash the executable locks Crimson and displays a full-screen integrity warning.

This detects a change relative to the current session. It does **not** prove that the downloaded executable was authentic before its first launch, and it cannot distinguish a malicious modification from a legitimate replacement without a trusted release identity.

## Why automatic file restoration is not enabled yet

A secure repair mechanism cannot trust a backup created by an unverified first-run executable. An attacker who replaces the downloaded EXE before first launch could cause the compromised program to bless its own files as the baseline.

A secure release needs all of the following before automatic repair can be enabled:

1. A code-signing identity for the published Windows executable.
2. A detached, digitally signed release manifest containing hashes for every installed application file and the recovery helper.
3. The manifest verification public key pinned in the application.
4. A separately signed recovery helper installed in a protected application directory with restrictive Windows ACLs.
5. A verified recovery payload that the normal application cannot silently rewrite.
6. A recovery test that deliberately corrupts a disposable test installation and proves that only Crimson-owned files are restored.
7. A repair log and a safe failure path when the signature, helper, or recovery payload cannot be verified.

The signing private key must remain outside the repository, preferably in a protected CI signing service or secret store. It must never be committed to Git.

## Recovery behavior once the release trust chain exists

- Verify the signed manifest before trusting any file.
- Check installed files on startup and periodically.
- On mismatch, stop normal Crimson operations and show the integrity warning.
- Ask the independent, signed recovery helper to restore only files listed in the verified manifest.
- Verify every restored file against the signed hashes.
- If recovery cannot be authenticated, leave the installation disabled and report the failure; never restore from an untrusted backup.
- Restart Crimson only after successful verification. Do not force-restart Windows or interrupt unrelated applications.

## Windows Defender and Firewall

Defender Antivirus remains the detection engine. Crimson may request a supported custom scan and report the result, but it does not attempt to control Defender's internal analysis. Windows Defender Firewall profiles and rules remain untouched.

## Explicit non-goals

- No self-deletion, destructive uninstall, or forced one-second reboot.
- No modification of GitHub source, releases, or other users' installations when a local copy is altered.
- No deletion or restoration of user documents, game files, or other applications.
- No claim that a hash captured from the running process is a trusted release signature.
