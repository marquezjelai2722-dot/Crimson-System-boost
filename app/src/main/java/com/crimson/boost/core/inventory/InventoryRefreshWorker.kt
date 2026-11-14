package com.crimson.boost.core.inventory

import android.content.Context
import androidx.work.Worker
import androidx.work.WorkerParameters

/**
 * Periodic metadata refresh only. This is not an antivirus scan and does not inspect
 * private app data. WorkManager controls execution timing and battery constraints.
 */
class InventoryRefreshWorker(
    appContext: Context,
    params: WorkerParameters
) : Worker(appContext, params) {
    override fun doWork(): Result = try {
        val report = InstalledAppInventory(applicationContext.packageManager).scan()
        applicationContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            .edit()
            .putLong(KEY_LAST_SCAN, report.generatedAtMillis)
            .putInt(KEY_APP_COUNT, report.apps.size)
            .putInt(KEY_SKIPPED_COUNT, report.skippedPackages)
            .apply()
        Result.success()
    } catch (error: Exception) {
        if (runAttemptCount < MAX_RETRIES) Result.retry() else Result.failure()
    }

    companion object {
        const val PREFS = "crimson_inventory"
        const val KEY_LAST_SCAN = "last_inventory_scan_ms"
        const val KEY_APP_COUNT = "inventory_app_count"
        const val KEY_SKIPPED_COUNT = "inventory_skipped_count"
        private const val MAX_RETRIES = 2
    }
}
