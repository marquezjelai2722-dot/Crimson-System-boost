package com.crimson.boost

import android.app.Activity
import android.os.Bundle
import android.os.Build
import android.content.Intent
import android.graphics.Color
import android.view.Gravity
import android.view.View
import android.widget.*
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL
import java.nio.charset.StandardCharsets
import java.util.concurrent.Executors

class MainActivity : Activity() {
    private val bg = Color.rgb(12, 16, 25)
    private val panel = Color.rgb(24, 31, 45)
    private val accent = Color.rgb(91, 220, 160)
    private val executor = Executors.newSingleThreadExecutor()
    private lateinit var chatLog: TextView
    private lateinit var endpointInput: EditText
    private lateinit var messageInput: EditText

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        render()
    }

    override fun onDestroy() {
        executor.shutdownNow()
        super.onDestroy()
    }

    private fun render() {
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(22, 30, 22, 20)
            setBackgroundColor(bg)
        }
        val scroll = ScrollView(this)
        val content = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        content.addView(label("CRIMSON", 13, accent))
        content.addView(label("SYSTEM BOOST", 28, Color.WHITE))
        content.addView(label("Device tools • AI assistant • v0.2.0", 13, 0xFFADB8C8.toInt()))
        content.addView(space(12))
        content.addView(card("DEVICE PROFILE", deviceSummary()))
        content.addView(action("Run device inventory") { showInventory() })
        content.addView(action("Open battery settings") {
            safeOpen(Intent(android.provider.Settings.ACTION_BATTERY_SAVER_SETTINGS))
        })
        content.addView(action("Open app settings") {
            safeOpen(Intent(android.provider.Settings.ACTION_MANAGE_APPLICATIONS_SETTINGS))
        })
        content.addView(space(10))
        content.addView(label("CRIMSON AI ASSISTANT", 20, Color.WHITE))
        content.addView(label("Connect a private backend that holds your Gemini API key. Never paste a Gemini key into this APK.", 13, 0xFFFFC66D.toInt()))
        endpointInput = EditText(this).apply {
            hint = "https://your-server.example.com/api/chat"
            setText(getPreferences(MODE_PRIVATE).getString("backend_url", ""))
            setTextColor(Color.WHITE)
            setHintTextColor(0xFF8D98A8.toInt())
            setSingleLine(true)
        }
        content.addView(endpointInput)
        content.addView(action("Save assistant server") {
            val value = endpointInput.text.toString().trim()
            if (value.startsWith("https://") && value.endsWith("/api/chat")) {
                getPreferences(MODE_PRIVATE).edit().putString("backend_url", value).apply()
                toast("Assistant server saved")
            } else {
                toast("Enter a valid HTTPS address ending in /api/chat")
            }
        })
        chatLog = label("Assistant is ready when a secure backend URL is configured.", 14, Color.WHITE).apply {
            setPadding(14, 14, 14, 14)
            setBackgroundColor(panel)
        }
        content.addView(chatLog)
        messageInput = EditText(this).apply {
            hint = "Ask about device settings or gaming..."
            setTextColor(Color.WHITE)
            setHintTextColor(0xFF8D98A8.toInt())
            minLines = 2
            gravity = Gravity.TOP
        }
        content.addView(messageInput)
        content.addView(action("Send to Crimson AI") { sendMessage() })
        content.addView(space(14))
        content.addView(card("GAMING MODE", "Quick access to Android settings. This app does not overclock hardware, bypass thermal limits, or promise unsupported FPS gains."))
        content.addView(card("FIREWALL", "Per-app network blocking requires a user-approved Android VPN service. Not enabled in this build."))
        content.addView(card("FILE SAFETY", "Future scanning is limited to user-selected files. Quarantine and deletion must be reviewable and user-confirmed."))
        content.addView(card("ADVANCED MODE • SHIZUKU", "Optional and not required for basic features. No privileged command runs without explicit permission and confirmation."))
        content.addView(label("No blind system-package removal. No thermal bypass. No guaranteed performance claims.", 12, 0xFFFFC66D.toInt()))
        scroll.addView(content)
        root.addView(scroll)
        setContentView(root)
    }

    private fun sendMessage() {
        val endpoint = endpointInput.text.toString().trim()
        val message = messageInput.text.toString().trim()
        if (!endpoint.startsWith("https://") || !endpoint.endsWith("/api/chat")) {
            toast("Save a valid HTTPS assistant server first")
            return
        }
        if (message.isBlank()) {
            toast("Type a message first")
            return
        }
        messageInput.isEnabled = false
        appendChat("\nYou: $message\nCrimson: Thinking…")
        messageInput.text.clear()
        executor.execute {
            val answer = try {
                val connection = (URL(endpoint).openConnection() as HttpURLConnection).apply {
                    requestMethod = "POST"
                    connectTimeout = 12000
                    readTimeout = 25000
                    doOutput = true
                    setRequestProperty("Content-Type", "application/json; charset=utf-8")
                }
                try {
                    val payload = JSONObject().put("message", message).toString()
                    connection.outputStream.use { it.write(payload.toByteArray(StandardCharsets.UTF_8)) }
                    val code = connection.responseCode
                    val stream = if (code in 200..299) connection.inputStream else connection.errorStream
                    val body = stream?.bufferedReader()?.use { it.readText() }.orEmpty()
                    if (code !in 200..299) {
                        val detail = try { JSONObject(body).optString("error", "Server error ($code)") } catch (_: Exception) { "Server error ($code)" }
                        "Request failed: $detail"
                    } else {
                        JSONObject(body).optString("reply").takeIf { it.isNotBlank() } ?: "The server returned an empty response."
                    }
                } finally {
                    connection.disconnect()
                }
            } catch (e: Exception) {
                "Couldn't reach the assistant server. Check the URL and connection, then try again."
            }
            runOnUiThread {
                chatLog.text = chatLog.text.toString().replace("Crimson: Thinking…", "Crimson: $answer")
                messageInput.isEnabled = true
            }
        }
    }

    private fun appendChat(value: String) { chatLog.text = chatLog.text.toString() + value }

    private fun showInventory() {
        try {
            val pm = packageManager
            val apps = if (Build.VERSION.SDK_INT >= 33) {
                pm.getInstalledApplications(android.content.pm.PackageManager.ApplicationInfoFlags.of(0))
            } else {
                @Suppress("DEPRECATION")
                pm.getInstalledApplications(0)
            }
            val userApps = apps.count { (it.flags and android.content.pm.ApplicationInfo.FLAG_SYSTEM) == 0 }
            appendChat("\n\nInventory: ${apps.size} visible packages; ${userApps} non-system apps.")
        } catch (_: Exception) {
            toast("Android did not allow the app inventory")
        }
    }

    private fun deviceSummary() =
        "Manufacturer: ${Build.MANUFACTURER}\nModel: ${Build.MODEL}\nDevice: ${Build.DEVICE}\nAndroid: ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})\nBuild: ${Build.DISPLAY}"

    private fun safeOpen(intent: Intent) {
        try { startActivity(intent) } catch (_: Exception) { toast("This settings page is unavailable on this device") }
    }

    private fun label(text: String, size: Int, color: Int) = TextView(this).apply {
        this.text = text
        textSize = size.toFloat()
        setTextColor(color)
        setPadding(0, 5, 0, 5)
    }

    private fun card(title: String, body: String) = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(16, 14, 16, 14)
        setBackgroundColor(panel)
        addView(label(title, 14, accent))
        addView(label(body, 14, Color.WHITE))
        layoutParams = LinearLayout.LayoutParams(-1, -2).apply { setMargins(0, 8, 0, 8) }
    }

    private fun action(text: String, run: () -> Unit) = Button(this).apply {
        this.text = text
        setOnClickListener { run() }
    }

    private fun space(height: Int) = View(this).apply {
        layoutParams = LinearLayout.LayoutParams(1, height)
    }

    private fun toast(message: String) = Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
}
