package com.crimson.boost

import android.app.Activity
import android.app.ActivityManager
import android.content.Intent
import android.graphics.Color
import android.net.Uri
import android.os.BatteryManager
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.StatFs
import android.provider.Settings
import android.view.Gravity
import android.view.View
import android.widget.*
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL
import java.nio.charset.StandardCharsets
import java.util.concurrent.Executors

class MainActivity : Activity() {
    private val bg = Color.rgb(10, 14, 23)
    private val panel = Color.rgb(23, 31, 45)
    private val accent = Color.rgb(255, 76, 91)
    private val muted = 0xFFADB8C8.toInt()
    private val executor = Executors.newSingleThreadExecutor()
    private val handler = Handler(Looper.getMainLooper())
    private lateinit var monitorText: TextView
    private lateinit var chatLog: TextView
    private lateinit var endpointInput: EditText
    private lateinit var messageInput: EditText
    private var monitorRunning = false

    private val monitorTick = object : Runnable {
        override fun run() {
            if (!monitorRunning) return
            monitorText.text = hardwareSummary()
            handler.postDelayed(this, 2000)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        render()
    }

    override fun onResume() {
        super.onResume()
        monitorRunning = true
        if (::monitorText.isInitialized) handler.post(monitorTick)
    }

    override fun onPause() {
        monitorRunning = false
        handler.removeCallbacks(monitorTick)
        super.onPause()
    }

    override fun onDestroy() {
        executor.shutdownNow()
        handler.removeCallbacksAndMessages(null)
        super.onDestroy()
    }

    private fun render() {
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(18, 24, 18, 18)
            setBackgroundColor(bg)
        }
        val scroll = ScrollView(this)
        val content = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        content.addView(label("CRIMSON / MOBILE UTILITIES", 12, accent))
        content.addView(label("SYSTEM BOOST", 30, Color.WHITE))
        content.addView(label("Device tools • gaming dashboard • AI assistant", 13, muted))
        content.addView(space(12))
        content.addView(card("DEVICE PROFILE", deviceSummary()))
        content.addView(label("LIVE HARDWARE MONITOR", 19, Color.WHITE))
        monitorText = label(hardwareSummary(), 14, Color.WHITE).apply {
            setPadding(14, 14, 14, 14)
            setBackgroundColor(panel)
        }
        content.addView(monitorText)
        content.addView(space(8))
        content.addView(label("QUICK TOOLS", 19, Color.WHITE))
        content.addView(action("Open battery settings") { safeOpen(Intent(Settings.ACTION_BATTERY_SAVER_SETTINGS)) })
        content.addView(action("Open display settings") { safeOpen(Intent(Settings.ACTION_DISPLAY_SETTINGS)) })
        content.addView(action("Open app management") { safeOpen(Intent(Settings.ACTION_MANAGE_APPLICATIONS_SETTINGS)) })
        content.addView(action("Open developer options") { safeOpen(Intent(Settings.ACTION_APPLICATION_DEVELOPMENT_SETTINGS)) })
        content.addView(action("Open storage settings") { safeOpen(Intent(Settings.ACTION_INTERNAL_STORAGE_SETTINGS)) })
        content.addView(action("Refresh installed-app inventory") { showInventory() })
        content.addView(action("Find launchable apps / games") { showLaunchableApps() })
        content.addView(space(10))
        content.addView(label("GAMING & PERFORMANCE", 19, Color.WHITE))
        content.addView(card("GAME MODE", "Use your phone's built-in game mode where available. Crimson provides safe shortcuts and monitoring; it does not claim to overclock the CPU/GPU or force unsupported FPS."))
        content.addView(card("DISPLAY TOOLS", "Open Android display settings to review refresh-rate options supported by your device. Unsupported refresh rates cannot be created by an app."))
        content.addView(card("NETWORK / PRIVATE DNS", "Open Network settings to configure Android Private DNS. DNS changes are user-controlled and can be reverted in Settings."))
        content.addView(action("Open network & internet settings") { safeOpen(Intent(Settings.ACTION_WIRELESS_SETTINGS)) })
        content.addView(card("ADVANCED MODE • SHIZUKU", "Shizuku integration is planned as an optional permission-gated module. No arbitrary shell scripts or destructive system commands are run in this preview."))
        content.addView(space(10))
        content.addView(label("CRIMSON AI ASSISTANT", 20, Color.WHITE))
        content.addView(label("Connect your own HTTPS backend. Keep Gemini keys on the server, never in the APK.", 13, 0xFFFFC66D.toInt()))
        endpointInput = EditText(this).apply {
            hint = "https://your-server.example.com/api/chat"
            setText(getPreferences(MODE_PRIVATE).getString("backend_url", ""))
            setTextColor(Color.WHITE); setHintTextColor(muted); setSingleLine(true)
        }
        content.addView(endpointInput)
        content.addView(action("Save assistant server") {
            val value = endpointInput.text.toString().trim()
            if (value.startsWith("https://") && value.endsWith("/api/chat")) {
                getPreferences(MODE_PRIVATE).edit().putString("backend_url", value).apply()
                toast("Assistant server saved")
            } else toast("Use an HTTPS URL ending in /api/chat")
        })
        chatLog = label("Assistant ready when a secure backend URL is configured.", 14, Color.WHITE).apply {
            setPadding(14, 14, 14, 14); setBackgroundColor(panel)
        }
        content.addView(chatLog)
        messageInput = EditText(this).apply {
            hint = "Ask about Android settings or gaming..."
            setTextColor(Color.WHITE); setHintTextColor(muted); minLines = 2; gravity = Gravity.TOP
        }
        content.addView(messageInput)
        content.addView(action("Send to Crimson AI") { sendMessage() })
        content.addView(space(16))
        content.addView(label("SAFETY", 13, 0xFFFFC66D.toInt()))
        content.addView(label("No blind system-app removal • no thermal-limit bypass • no guaranteed FPS claims • no hidden API-key storage.", 12, muted))
        scroll.addView(content)
        root.addView(scroll)
        setContentView(root)
    }

    private fun hardwareSummary(): String {
        val am = getSystemService(ACTIVITY_SERVICE) as ActivityManager
        val memory = ActivityManager.MemoryInfo()
        am.getMemoryInfo(memory)
        val stat = StatFs(filesDir.absolutePath)
        val totalStorage = stat.totalBytes / (1024L * 1024L * 1024L)
        val freeStorage = stat.availableBytes / (1024L * 1024L * 1024L)
        val battery = (getSystemService(BATTERY_SERVICE) as BatteryManager)
            .getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
        val refresh = if (Build.VERSION.SDK_INT >= 30) display?.refreshRate ?: 0f else @Suppress("DEPRECATION") windowManager.defaultDisplay.refreshRate
        val usedRam = (memory.totalMem - memory.availMem) / (1024L * 1024L)
        val totalRam = memory.totalMem / (1024L * 1024L)
        return "RAM: ${usedRam} / ${totalRam} MB\nStorage: ${freeStorage} GB free / ${totalStorage} GB total\nBattery: ${if (battery >= 0) "$battery%" else "Unavailable"}\nDisplay refresh: ${"%.1f".format(refresh)} Hz\nDevice: ${Build.MODEL} • Android ${Build.VERSION.RELEASE}"
    }

    private fun showLaunchableApps() {
        val pm = packageManager
        val intent = Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER)
        val activities = if (Build.VERSION.SDK_INT >= 33) pm.queryIntentActivities(intent, android.content.pm.PackageManager.ResolveInfoFlags.of(0)) else @Suppress("DEPRECATION") pm.queryIntentActivities(intent, 0)
        val choices = activities.distinctBy { it.activityInfo.packageName }.sortedBy { it.loadLabel(pm).toString().lowercase() }
        if (choices.isEmpty()) { toast("No launchable apps found"); return }
        val names = choices.map { it.loadLabel(pm).toString() }.toTypedArray()
        AlertDialogBuilder.show(this, "Launch an app", names) { index ->
            val launch = pm.getLaunchIntentForPackage(choices[index].activityInfo.packageName)
            if (launch != null) safeOpen(launch)
        }
    }

    private fun showInventory() {
        try {
            val pm = packageManager
            val apps = if (Build.VERSION.SDK_INT >= 33) pm.getInstalledApplications(android.content.pm.PackageManager.ApplicationInfoFlags.of(0)) else @Suppress("DEPRECATION") pm.getInstalledApplications(0)
            val userApps = apps.count { (it.flags and android.content.pm.ApplicationInfo.FLAG_SYSTEM) == 0 }
            Toast.makeText(this, "${apps.size} visible packages • ${userApps} user apps", Toast.LENGTH_LONG).show()
        } catch (_: Exception) { toast("Android did not allow app inventory access") }
    }

    private fun sendMessage() {
        val endpoint = endpointInput.text.toString().trim()
        val message = messageInput.text.toString().trim()
        if (!endpoint.startsWith("https://") || !endpoint.endsWith("/api/chat")) { toast("Save a valid HTTPS server first"); return }
        if (message.isBlank()) { toast("Type a message first"); return }
        messageInput.isEnabled = false
        val marker = "\nYou: $message\nCrimson: Thinking…"
        chatLog.append(marker)
        messageInput.text.clear()
        executor.execute {
            val answer = try {
                val connection = (URL(endpoint).openConnection() as HttpURLConnection).apply {
                    requestMethod = "POST"; connectTimeout = 12000; readTimeout = 25000; doOutput = true
                    setRequestProperty("Content-Type", "application/json; charset=utf-8")
                }
                try {
                    val payload = JSONObject().put("message", message).toString()
                    connection.outputStream.use { it.write(payload.toByteArray(StandardCharsets.UTF_8)) }
                    val code = connection.responseCode
                    val body = (if (code in 200..299) connection.inputStream else connection.errorStream)?.bufferedReader()?.use { it.readText() }.orEmpty()
                    if (code !in 200..299) {
                        val detail = try { JSONObject(body).optString("error", "HTTP $code") } catch (_: Exception) { "HTTP $code" }
                        "Request failed: $detail"
                    } else JSONObject(body).optString("reply").ifBlank { "The server returned an empty reply." }
                } finally { connection.disconnect() }
            } catch (_: Exception) { "Couldn't reach the server. Check your connection and backend URL." }
            runOnUiThread {
                chatLog.text = chatLog.text.toString().replace("Crimson: Thinking…", "Crimson: $answer")
                messageInput.isEnabled = true
            }
        }
    }

    private fun deviceSummary() = "Manufacturer: ${Build.MANUFACTURER}\nModel: ${Build.MODEL}\nDevice: ${Build.DEVICE}\nAndroid: ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})\nBuild: ${Build.DISPLAY}"
    private fun safeOpen(intent: Intent) { try { startActivity(intent) } catch (_: Exception) { toast("This settings page is unavailable") } }
    private fun label(text: String, size: Int, color: Int) = TextView(this).apply { this.text=text; textSize=size.toFloat(); setTextColor(color); setPadding(0,5,0,5) }
    private fun card(title: String, body: String) = LinearLayout(this).apply {
        orientation=LinearLayout.VERTICAL; setPadding(15,13,15,13); setBackgroundColor(panel)
        addView(label(title,14,accent)); addView(label(body,14,Color.WHITE))
        layoutParams=LinearLayout.LayoutParams(-1,-2).apply { setMargins(0,7,0,7) }
    }
    private fun action(text: String, run: () -> Unit) = Button(this).apply { this.text=text; setOnClickListener { run() } }
    private fun space(height: Int) = View(this).apply { layoutParams=LinearLayout.LayoutParams(1,height) }
    private fun toast(message: String) = Toast.makeText(this,message,Toast.LENGTH_SHORT).show()
}

private object AlertDialogBuilder {
    fun show(activity: Activity, title: String, items: Array<String>, onPick: (Int) -> Unit) {
        android.app.AlertDialog.Builder(activity).setTitle(title).setItems(items) { _, which -> onPick(which) }.setNegativeButton("Cancel", null).show()
    }
}
