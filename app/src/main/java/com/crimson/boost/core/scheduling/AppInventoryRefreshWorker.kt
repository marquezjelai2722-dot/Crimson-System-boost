package com.crimson.boost.core.scheduling

import android.content.Context
import androidx.work.Worker
import androidx.work.WorkerParameters
import com.crimson.boost.core.inventory.InstalledAppScanner
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

/**
 * OS-scheduled metadata refresh. WorkManager decides when this runs; it is not
 * a continuously running antivirus or a private-file scanner.
 */
class AppInventoryRefreshWorker(
    context: Context,
    parameters: WorkerParameters
) : Worker(context, parameters) {
    override fun doWork(): Result {
        return try {
            val report = InstalledAppScanner(applicationContext).scan()
            val apps = JSONArray()
            report.apps.forEach { app ->
                apps.put(
                    JSONObject()
                        .put("packageName", app.packageName)
                        .put("displayName", app.displayName)
                        .put("classification", app.classification.name)
                        .put("signerSha256", JSONArray(app.signerSha256))
                )
            }
            val output = JSONObject()
                .put("schemaVersion", 1)
                .put("completedAtEpochMs", report.completedAtEpochMs)
                .put("durationMs", report.durationMs)
                .put("appCount", report.apps.size)
                .put("warnings", JSONArray(report.warnings))
                .put("apps", apps)
            val directory = File(applicationContext.filesDir, "inventory")
            if (!directory.exists() && !directory.mkdirs()) {
                return Result.retry()
            }
            val target = File(directory, "latest-inventory.json")
            val temporary = File(directory, "latest-inventory.json.tmp")
            temporary.writeText(output.toString(), Charsets.UTF_8)
            if (target.exists() && !target.delete()) return Result.retry()
            if (!temporary.renameTo(target)) return Result.retry()
            Result.success()
        } catch (_: SecurityException) {
            Result.failure()
        } catch (_: Exception) {
            Result.retry()
        }
    }
}
