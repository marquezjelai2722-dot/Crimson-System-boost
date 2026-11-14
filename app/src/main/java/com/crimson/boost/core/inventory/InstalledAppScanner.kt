package com.crimson.boost.core.inventory

import android.content.Context
import android.content.pm.ApplicationInfo
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.os.Build
import com.crimson.boost.core.model.AppClassification
import com.crimson.boost.core.model.InstalledAppRecord
import com.crimson.boost.core.model.InventoryReport
import java.security.MessageDigest

/**
 * Reads public package metadata only. It does not inspect another app's private
 * storage, runtime memory, network traffic, or code for malware.
 */
class InstalledAppScanner(private val context: Context) {
    fun scan(): InventoryReport {
        val started = System.currentTimeMillis()
        val warnings = mutableListOf<String>()
        val records = mutableListOf<InstalledAppRecord>()
        val pm = context.packageManager

        val installed = try {
            if (Build.VERSION.SDK_INT >= 33) {
                pm.getInstalledApplications(PackageManager.ApplicationInfoFlags.of(0L))
            } else {
                @Suppress("DEPRECATION")
                pm.getInstalledApplications(0)
            }
        } catch (_: SecurityException) {
            warnings += "Package inventory access was restricted."
            emptyList()
        } catch (error: RuntimeException) {
            warnings += "Package inventory failed: ${error.javaClass.simpleName}"
            emptyList()
        }

        for (app in installed) {
            try {
                val info = packageInfo(pm, app.packageName)
                val system = (app.flags and ApplicationInfo.FLAG_SYSTEM) != 0 ||
                    (app.flags and ApplicationInfo.FLAG_UPDATED_SYSTEM_APP) != 0
                val enabled = app.enabled
                val launchable = pm.getLaunchIntentForPackage(app.packageName) != null
                val classification = when {
                    !enabled -> AppClassification.DISABLED
                    system -> AppClassification.PREINSTALLED_REVIEW
                    else -> AppClassification.USER_INSTALLED
                }
                records += InstalledAppRecord(
                    packageName = app.packageName,
                    displayName = app.loadLabel(pm).toString().take(160)
                        .ifBlank { app.packageName },
                    versionName = info?.versionName,
                    firstInstallTime = info?.firstInstallTime,
                    lastUpdateTime = info?.lastUpdateTime,
                    isSystemPackage = system,
                    isEnabled = enabled,
                    isLaunchable = launchable,
                    signerSha256 = signerDigests(info),
                    classification = classification
                )
            } catch (_: SecurityException) {
                warnings += "Some package metadata was unavailable."
            } catch (_: RuntimeException) {
                warnings += "One package was skipped because metadata could not be read."
            }
        }

        return InventoryReport(
            startedAtEpochMs = started,
            completedAtEpochMs = System.currentTimeMillis(),
            apps = records.sortedBy { it.displayName.lowercase() },
            warnings = warnings.distinct()
        )
    }

    private fun packageInfo(pm: PackageManager, packageName: String): PackageInfo? = try {
        if (Build.VERSION.SDK_INT >= 33) {
            val flags = if (Build.VERSION.SDK_INT >= 28) {
                PackageManager.GET_SIGNING_CERTIFICATES.toLong()
            } else {
                @Suppress("DEPRECATION")
                PackageManager.GET_SIGNATURES.toLong()
            }
            pm.getPackageInfo(packageName, PackageManager.PackageInfoFlags.of(flags))
        } else {
            @Suppress("DEPRECATION")
            pm.getPackageInfo(
                packageName,
                if (Build.VERSION.SDK_INT >= 28) PackageManager.GET_SIGNING_CERTIFICATES
                else PackageManager.GET_SIGNATURES
            )
        }
    } catch (_: PackageManager.NameNotFoundException) {
        null
    } catch (_: SecurityException) {
        null
    }

    private fun signerDigests(info: PackageInfo?): List<String> {
        val signatures = if (Build.VERSION.SDK_INT >= 28) {
            val signing = info?.signingInfo ?: return emptyList()
            if (signing.hasMultipleSigners()) signing.apkContentsSigners
            else signing.signingCertificateHistory
        } else {
            @Suppress("DEPRECATION")
            info?.signatures ?: return emptyList()
        }
        return signatures.mapNotNull { signature ->
            runCatching {
                MessageDigest.getInstance("SHA-256")
                    .digest(signature.toByteArray())
                    .joinToString("") { byte -> "%02x".format(byte) }
            }.getOrNull()
        }.distinct()
    }
}
