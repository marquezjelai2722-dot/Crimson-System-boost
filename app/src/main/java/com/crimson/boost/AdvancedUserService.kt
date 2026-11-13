package com.crimson.boost

import android.os.RemoteException
import java.util.concurrent.TimeUnit

/**
 * Runs inside Shizuku's UserService process (shell/ADB identity).
 * Only fixed, allow-listed Android package-manager and game-manager operations
 * are exposed to the app. No arbitrary command or shell-string input is accepted.
 */
class AdvancedUserService : IAdvancedService.Stub() {

    override fun setGameMode(packageName: String, mode: String): String {
        if (!isPackageName(packageName)) return "Rejected: invalid package name."
        if (mode != "performance" && mode != "standard") {
            return "Rejected: unsupported game mode."
        }
        return execute(
            listOf("/system/bin/cmd", "game", "mode", mode, packageName),
            35
        )
    }

    override fun trimSystemCaches(): String {
        // Trims reclaimable app caches; it does not clear app data or user files.
        return execute(listOf("/system/bin/pm", "trim-caches", "256M"), 90)
    }

    override fun compileSelectedApp(packageName: String): String {
        if (!isPackageName(packageName)) return "Rejected: invalid package name."
        // This asks ART to compile the selected package using its available profile.
        return execute(
            listOf("/system/bin/cmd", "package", "compile", "-m", "speed-profile", "-f", packageName),
            120
        )
    }

    private fun isPackageName(value: String): Boolean =
        value.length in 3..255 &&
            value.matches(Regex("[A-Za-z0-9_]+(\\.[A-Za-z0-9_]+)+"))

    private fun execute(command: List<String>, timeoutSeconds: Long): String {
        return try {
            val process = ProcessBuilder(command).redirectErrorStream(true).start()
            val finished = process.waitFor(timeoutSeconds, TimeUnit.SECONDS)
            if (!finished) {
                process.destroyForcibly()
                return "Timed out. The device may not support this operation."
            }
            val output = process.inputStream.bufferedReader().use { it.readText() }.trim()
            val status = process.exitValue()
            val detail = output.ifBlank { if (status == 0) "Command completed." else "No system output." }
            "Exit code $status\n$detail".take(1800)
        } catch (error: Exception) {
            "Operation unavailable: ${error.javaClass.simpleName}: ${error.message ?: "unknown error"}"
        }
    }
}
