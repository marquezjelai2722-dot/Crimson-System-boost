package com.crimson.boost.core.inventory

import android.content.pm.ApplicationInfo
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.os.Build
import java.io.File

enum class AppOrigin { SYSTEM, USER }
enum class InstallFootprint { SMALL, LARGE, UNKNOWN }
enum class SecurityVerdict { NOT_SCANNED, SUSPICIOUS, CONFIRMED_THREAT }

/**
 * Inventory metadata only. A large APK or a system package is not a malware verdict.
 * The current implementation deliberately returns NOT_SCANNED until a trusted
 * on-device reputation/signature engine is integrated.
 */
data class InstalledAppRecord(
    val packageName: String,
    val displayName: String,
    val versionName: String?,
    val origin: AppOrigin,
    val apkSizeBytes: Long?,
    val footprint: InstallFootprint,
    val verdict: SecurityVerdict = SecurityVerdict.NOT_SCANNED,
    val warning: String? = null
)

data class InventoryReport(
    val apps: List<InstalledAppRecord>,
    val skippedPackages: Int,
    val generatedAtMillis: Long
) {
    val systemApps: Int get() = apps.count { it.origin == AppOrigin.SYSTEM }
    val userApps: Int get() = apps.count { it.origin == AppOrigin.USER }
    val largeApks: Int get() = apps.count { it.footprint == InstallFootprint.LARGE }
}

/** Each package is isolated: a broken PackageInfo or unreadable APK never aborts the scan. */
class InstalledAppInventory(private val packageManager: PackageManager) {
    fun scan(): InventoryReport {
        val packages = if (Build.VERSION.SDK_INT >= 33) {
            packageManager.getInstalledPackages(PackageManager.PackageInfoFlags.of(0L))
        } else {
            @Suppress("DEPRECATION")
            packageManager.getInstalledPackages(0)
        }
        var skipped = 0
        val records = packages.mapNotNull { info ->
            try {
                toRecord(info)
            } catch (_: Exception) {
                skipped += 1
                null
            }
        }.sortedBy { it.displayName.lowercase() }

        return InventoryReport(records, skipped, System.currentTimeMillis())
    }

    private fun toRecord(info: PackageInfo): InstalledAppRecord {
        val appInfo = requireNotNull(info.applicationInfo)
        val label = runCatching { packageManager.getApplicationLabel(appInfo).toString() }
            .getOrDefault(info.packageName)
            .ifBlank { info.packageName }
        val system = appInfo.flags and
            (ApplicationInfo.FLAG_SYSTEM or ApplicationInfo.FLAG_UPDATED_SYSTEM_APP) != 0
        val apkBytes = runCatching {
            File(appInfo.sourceDir).takeIf { it.isFile }?.length()
        }.getOrNull()
        val footprint = when {
            apkBytes == null || apkBytes <= 0L -> InstallFootprint.UNKNOWN
            apkBytes >= LARGE_APK_THRESHOLD_BYTES -> InstallFootprint.LARGE
            else -> InstallFootprint.SMALL
        }
        return InstalledAppRecord(
            packageName = info.packageName,
            displayName = label,
            versionName = info.versionName,
            origin = if (system) AppOrigin.SYSTEM else AppOrigin.USER,
            apkSizeBytes = apkBytes,
            footprint = footprint
        )
    }

    companion object {
        const val LARGE_APK_THRESHOLD_BYTES = 100L * 1024L * 1024L
    }
}
