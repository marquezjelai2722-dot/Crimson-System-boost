package com.crimson.boost.core.model

/** Metadata-only classification. None of these values is an antivirus verdict. */
enum class AppClassification {
    PREINSTALLED_REVIEW,
    USER_INSTALLED,
    DISABLED,
    UNKNOWN
}

data class InstalledAppRecord(
    val packageName: String,
    val displayName: String,
    val versionName: String?,
    val firstInstallTime: Long?,
    val lastUpdateTime: Long?,
    val isSystemPackage: Boolean,
    val isEnabled: Boolean,
    val isLaunchable: Boolean,
    val signerSha256: List<String>,
    val classification: AppClassification
)

data class InventoryReport(
    val startedAtEpochMs: Long,
    val completedAtEpochMs: Long,
    val apps: List<InstalledAppRecord>,
    val warnings: List<String>
) {
    val durationMs: Long
        get() = (completedAtEpochMs - startedAtEpochMs).coerceAtLeast(0L)

    val preinstalledReviewCount: Int
        get() = apps.count { it.classification == AppClassification.PREINSTALLED_REVIEW }

    val disabledCount: Int
        get() = apps.count { !it.isEnabled }
}
