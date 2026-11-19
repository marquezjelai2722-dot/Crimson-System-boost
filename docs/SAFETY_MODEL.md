# Safety Model

## Required preflight for every system change

1. Identify the exact Windows version, edition, build, hardware and policy constraints.
2. Check that the target setting exists and is supported on this system.
3. Explain the expected effect, limitations and risk in plain language.
4. Create a Windows restore point and verify that Windows accepted it.
5. Save the exact original setting in an application snapshot.
6. Ask for explicit confirmation of the proposed change.
7. Apply only that one change or the explicitly reviewed batch.
8. Read the setting back and verify the result.
9. Record the result and provide a tested undo action.

If steps 1–5 fail, do not apply the change. If post-change verification fails, attempt the documented rollback and report the outcome.

## Permanently protected components

The application must not offer disable/remove controls for:
- Microsoft Defender and its security services;
- Windows Firewall;
- Windows Update;
- boot-critical storage, networking and device drivers;
- essential Windows services and security infrastructure.

## Prohibited blanket tweaks

Do not apply blind registry packs, timer-resolution forcing, core-parking changes, service-disabling lists, forced HAGS settings, or security-feature removal. Performance changes must be measured on the target system and be reversible.

## Restore points are not file backups

System Restore does not replace backups of personal documents, projects, or recordings. The app must say this before a user relies on restore functionality.
