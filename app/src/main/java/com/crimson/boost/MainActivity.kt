package com.crimson.boost

import android.app.Activity
import android.app.AlertDialog
import android.app.ActivityManager
import android.content.ComponentName
import android.content.Intent
import android.content.ServiceConnection
import android.content.pm.PackageManager
import android.content.pm.ResolveInfo
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.net.Uri
import android.os.BatteryManager
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.os.StatFs
import android.view.Gravity
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.FrameLayout
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.widget.Toast
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import org.json.JSONObject
import rikka.shizuku.Shizuku
import java.net.HttpURLConnection
import java.net.URL
import java.nio.charset.StandardCharsets
import java.util.concurrent.Executors
import com.crimson.boost.core.inventory.InstalledAppScanner
import com.crimson.boost.core.scheduling.AppInventoryRefreshWorker
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import java.util.concurrent.TimeUnit

class MainActivity : Activity() {
    private val bg = Color.rgb(10, 8, 14)
    private val surface = Color.rgb(22, 17, 27)
    private val surfaceAlt = Color.rgb(33, 26, 39)
    private val accent = Color.rgb(255, 59, 82)
    private val accentCool = Color.rgb(56, 217, 255)
    private val textMain = Color.rgb(245, 241, 246)
    private val textMuted = Color.rgb(174, 164, 180)
    private val success = Color.rgb(91, 205, 132)
    private val warning = Color.rgb(242, 177, 77)

    private val executor = Executors.newSingleThreadExecutor()
    private val handler = Handler(Looper.getMainLooper())
    private lateinit var monitorText: TextView
    private lateinit var shizukuStatus: TextView
    private lateinit var chatLog: TextView
    private lateinit var endpointInput: EditText
    private lateinit var messageInput: EditText
    private var monitorRunning = false
    private var advancedService: IAdvancedService? = null
    private var pendingAdvancedAction: ((IAdvancedService) -> String)? = null
    private var serviceBound = false

    private val permissionListener = Shizuku.OnRequestPermissionResultListener { requestCode, grantResult ->
        if (requestCode == SHIZUKU_REQUEST_CODE) {
            if (grantResult == PackageManager.PERMISSION_GRANTED) {
                bindAdvancedService()
            } else {
                toast("Shizuku permission was not granted.")
                updateShizukuStatus()
            }
        }
    }

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, binder: IBinder?) {
            advancedService = binder?.let { IAdvancedService.Stub.asInterface(it) }
            val action = pendingAdvancedAction
            pendingAdvancedAction = null
            val service = advancedService
            if (service != null && action != null) executeAdvanced(action, service)
            updateShizukuStatus()
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            advancedService = null
            serviceBound = false
            updateShizukuStatus()
        }
    }

    private val userServiceArgs by lazy {
        Shizuku.UserServiceArgs(
            ComponentName(packageName, AdvancedUserService::class.java.name)
        )
            .daemon(false)
            .processNameSuffix("crimson_advanced")
            .debuggable(BuildConfig.DEBUG)
            .version(BuildConfig.VERSION_CODE)
    }

    private val monitorTick = object : Runnable {
        override fun run() {
            if (!monitorRunning) return
            if (::monitorText.isInitialized) monitorText.text = hardwareSummary()
            handler.postDelayed(this, 2500)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        window.statusBarColor = bg
        window.navigationBarColor = bg
        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowCompat.getInsetsController(window, window.decorView).isAppearanceLightStatusBars = false
        WindowCompat.getInsetsController(window, window.decorView).isAppearanceLightNavigationBars = false
        Shizuku.addRequestPermissionResultListener(permissionListener)
        scheduleInventoryRefresh()
        showLoadingScreen()
        handler.postDelayed({ if (!isFinishing) render() }, 1100)
    }

    override fun onResume() {
        super.onResume()
        monitorRunning = true
        if (::monitorText.isInitialized) handler.post(monitorTick)
        updateShizukuStatus()
    }

    override fun onPause() {
        monitorRunning = false
        handler.removeCallbacks(monitorTick)
        super.onPause()
    }

    override fun onDestroy() {
        Shizuku.removeRequestPermissionResultListener(permissionListener)
        if (serviceBound) {
            try { Shizuku.unbindUserService(userServiceArgs, serviceConnection, true) } catch (_: Exception) {}
        }
        executor.shutdownNow()
        handler.removeCallbacksAndMessages(null)
        super.onDestroy()
    }

    private fun render() {
        val rootLeft = dp(18)
        val rootTop = dp(20)
        val rootRight = dp(18)
        val rootBottom = dp(18)
        val root = FrameLayout(this).apply {
            setPadding(rootLeft, rootTop, rootRight, rootBottom)
            setBackgroundColor(bg)
            clipChildren = false
            clipToPadding = false
        }
        ViewCompat.setOnApplyWindowInsetsListener(root) { view, insets ->
            val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
            val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
            view.setPadding(
                rootLeft + bars.left,
                rootTop + bars.top,
                rootRight + bars.right,
                rootBottom + maxOf(bars.bottom, ime.bottom)
            )
            insets
        }
        ViewCompat.requestApplyInsets(root)
        val scroll = ScrollView(this).apply { isFillViewport = true }
        val content = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, 0, 0, dp(12))
        }

        val header = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }
        header.addView(ImageView(this).apply {
            setImageResource(R.drawable.crimson_mark)
            layoutParams = LinearLayout.LayoutParams(dp(48), dp(48))
        })
        val brandText = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(12), 0, 0, 0)
        }
        brandText.addView(label("CRIMSON", 21, textMain, true))
        brandText.addView(label("SYSTEM BOOST  /  DEVICE CONTROL", 10, textMuted, true))
        header.addView(brandText)
        header.addView(View(this), LinearLayout.LayoutParams(0, 1, 1f))
        header.addView(label("0.3.0", 11, textMuted, true))
        content.addView(header)
        content.addView(space(18))

        val heroCard = cardContainer()
        heroCard.addView(label("CRIMSON  /  PERFORMANCE CONSOLE", 11, accentCool, true))
        heroCard.addView(label("Play better. Run smoother.", 23, textMain, true))
        heroCard.addView(label("Real device telemetry and permission-gated controls. No fake FPS promises.", 13, textMuted))
        val heroMeta = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; gravity = Gravity.CENTER_VERTICAL }
        heroMeta.addView(label("●  LIVE DEVICE MONITOR", 10, success, true))
        heroMeta.addView(View(this), LinearLayout.LayoutParams(0, 1, 1f))
        heroMeta.addView(label("ANDROID  /  0.3.0", 10, textMuted, true))
        heroCard.addView(space(5))
        heroCard.addView(heroMeta)
        content.addView(heroCard)
        content.addView(space(12))
        content.addView(label("ANDROID PERFORMANCE UTILITY", 11, accent, true))
        content.addView(label("System-level tools. Explicit control.", 22, textMain, true))
        content.addView(label("Local monitoring and permission-gated maintenance. No fake FPS promises.", 13, textMuted))
        content.addView(space(16))

        content.addView(sectionTitle("DEVICE STATUS"))
        val statusCard = cardContainer()
        monitorText = label(hardwareSummary(), 14, textMain).apply {
            typeface = Typeface.MONOSPACE
            setLineSpacing(dp(3).toFloat(), 1f)
        }
        statusCard.addView(monitorText)
        content.addView(statusCard)
        content.addView(space(12))

        content.addView(sectionTitle("ADVANCED CONTROL"))
        val shizukuCard = cardContainer()
        shizukuStatus = label("SHIZUKU STATUS: CHECKING", 12, warning, true)
        shizukuCard.addView(shizukuStatus)
        shizukuCard.addView(label(
            "Advanced operations run through Shizuku's UserService with ADB/shell identity. Each action is started manually and reports the system command result.",
            13, textMuted
        ))
        shizukuCard.addView(space(8))
        shizukuCard.addView(primaryAction("Connect Shizuku") { connectShizuku() })
        shizukuCard.addView(secondaryAction("Open Shizuku manager") { openShizukuManager() })
        content.addView(shizukuCard)
        content.addView(space(8))

        val gameCard = cardContainer()
        gameCard.addView(label("GAME MODE", 14, textMain, true))
        gameCard.addView(label(
            "Request Android GameManager performance mode for one selected game. Availability depends on Android version, OEM support, and whether the package is recognized as a game.",
            13, textMuted
        ))
        gameCard.addView(primaryAction("Set performance mode for a game") { chooseGame { pkg, title ->
            confirm(
                "Enable performance mode?",
                "Crimson will request Android GameManager performance mode for $title. This may increase power use and heat. The device can reject or ignore the request.",
                "Apply"
            ) { runAdvanced("Game mode: $title") { it.setGameMode(pkg, "performance") } }
        } })
        gameCard.addView(secondaryAction("Restore standard mode for a game") { chooseGame { pkg, title ->
            confirm(
                "Restore standard mode?",
                "This requests Android's standard game mode for $title.",
                "Restore"
            ) { runAdvanced("Standard game mode: $title") { it.setGameMode(pkg, "standard") } }
        } })
        content.addView(gameCard)
        content.addView(space(8))

        val maintenanceCard = cardContainer()
        maintenanceCard.addView(label("SYSTEM CACHE MAINTENANCE", 14, textMain, true))
        maintenanceCard.addView(label(
            "Ask Android Package Manager to trim reclaimable application caches until its 256 MB target is met. App data, accounts, photos, and downloads are not targeted. Cached content may be rebuilt later.",
            13, textMuted
        ))
        maintenanceCard.addView(secondaryAction("Review and trim caches") {
            confirm(
                "Trim reclaimable caches?",
                "This affects reclaimable caches across apps. It cannot restore deleted cache files, but it does not request app-data or personal-file deletion.",
                "Trim caches"
            ) { runAdvanced("System cache maintenance") { it.trimSystemCaches() } }
        })
        content.addView(maintenanceCard)
        content.addView(space(8))

        val compileCard = cardContainer()
        compileCard.addView(label("APP RUNTIME OPTIMIZER", 14, textMain, true))
        compileCard.addView(label(
            "Request Android Runtime profile-guided compilation for one selected app. This can take time, use storage, or be unsupported; it is not a guaranteed FPS increase.",
            13, textMuted
        ))
        compileCard.addView(secondaryAction("Optimize one app runtime") { chooseGame { pkg, title ->
            confirm(
                "Compile $title?",
                "Android will compile this app using its available speed profile. It may take a while and the system may report that no profile or operation is available.",
                "Compile"
            ) { runAdvanced("Runtime compilation: $title") { it.compileSelectedApp(pkg) } }
        } })
        content.addView(compileCard)
        content.addView(space(16))

        content.addView(sectionTitle("APP TOOLS"))
        val appCard = cardContainer()
        appCard.addView(label("LOCAL APP LAUNCHER", 14, textMain, true))
        appCard.addView(label("Browse launchable apps and games without sending the inventory to a server.", 13, textMuted))
        appCard.addView(secondaryAction("Browse installed launchable apps") { chooseGame { pkg, title ->
            val launch = packageManager.getLaunchIntentForPackage(pkg)
            if (launch != null) safeOpen(launch) else toast("$title has no launcher activity.")
        } })
        content.addView(appCard)
        content.addView(space(8))

        val inventoryCard = cardContainer()
        inventoryCard.addView(label("INSTALLED APP INVENTORY", 14, textMain, true))
        inventoryCard.addView(label(
            "Review preinstalled and user-installed app metadata. This is not an antivirus scan and does not inspect other apps' private files.",
            13, textMuted
        ))
        val inventoryResult = label("Inventory has not been scanned yet.", 12, textMuted)
        inventoryCard.addView(inventoryResult)
        inventoryCard.addView(secondaryAction("Scan installed app metadata") {
            inventoryResult.text = "Scanning installed package metadata…"
            executor.execute {
                val report = InstalledAppScanner(this).scan()
                handler.post {
                    inventoryResult.text = "Found ${report.apps.size} packages • ${report.preinstalledReviewCount} preinstalled review candidates • ${report.disabledCount} disabled"
                    val preview = report.apps.take(18).joinToString("\n") {
                        "${it.displayName} — ${it.classification.name.replace('_', ' ')}"
                    }
                    val warnings = if (report.warnings.isEmpty()) "" else "\n\nWarnings: ${report.warnings.joinToString("; ")}"
                    AlertDialog.Builder(this)
                        .setTitle("Installed app inventory")
                        .setMessage("${preview.ifBlank { "No package metadata was available." }}${warnings}\n\nPreinstalled does not mean malicious. Use your device's security protection for malware verdicts.")
                        .setPositiveButton("Close", null)
                        .show()
                }
            }
        })
        content.addView(inventoryCard)
        content.addView(space(16))

        content.addView(sectionTitle("CRIMSON ASSISTANT"))
        val assistantCard = cardContainer()
        assistantCard.addView(label(
            "Optional HTTPS backend. Keep provider API keys on the server; never place them in the APK.",
            13, textMuted
        ))
        endpointInput = EditText(this).apply {
            hint = "https://your-server.example.com/api/chat"
            setText(getPreferences(MODE_PRIVATE).getString("backend_url", ""))
            setTextColor(textMain)
            setHintTextColor(textMuted)
            setSingleLine(true)
            textSize = 13f
            background = roundedBackground(surfaceAlt, Color.rgb(75, 55, 70), 12)
            setPadding(dp(12), dp(10), dp(12), dp(10))
        }
        assistantCard.addView(endpointInput)
        assistantCard.addView(secondaryAction("Save assistant endpoint") {
            val value = endpointInput.text.toString().trim()
            if (value.startsWith("https://") && value.endsWith("/api/chat")) {
                getPreferences(MODE_PRIVATE).edit().putString("backend_url", value).apply()
                toast("HTTPS endpoint saved.")
            } else toast("Use HTTPS and end the URL with /api/chat.")
        })
        chatLog = label("Assistant idle. Configure an HTTPS backend to enable requests.", 13, textMain).apply {
            setPadding(dp(10), dp(10), dp(10), dp(10))
            background = roundedBackground(surfaceAlt, Color.rgb(75, 55, 70), 12)
        }
        assistantCard.addView(chatLog)
        messageInput = EditText(this).apply {
            hint = "Enter a question"
            setTextColor(textMain)
            setHintTextColor(textMuted)
            minLines = 2
            gravity = Gravity.TOP
            setBackgroundColor(surfaceAlt)
        }
        assistantCard.addView(messageInput)
        assistantCard.addView(primaryAction("Send request") { sendMessage() })
        content.addView(assistantCard)
        content.addView(space(16))
        content.addView(label(
            "SAFETY MODEL  •  No thermal-limit bypass  •  No overclocking  •  No blind package removal  •  No arbitrary shell input",
            11, textMuted
        ))

        scroll.addView(content)
        root.addView(scroll, FrameLayout.LayoutParams(-1, -1))
        val mascot = CrimsonMascotView(this).apply {
            contentDescription = "Crimson mascot. Drag to move, tap for tutorial."
            elevation = dp(18).toFloat()
            setOnClickListener { showMascotTutorial() }
        }
        val mascotParams = FrameLayout.LayoutParams(dp(104), dp(150), Gravity.BOTTOM or Gravity.END).apply {
            rightMargin = dp(12)
            bottomMargin = dp(20)
        }
        mascot.setOnLongClickListener {
            mascot.setFolderPose(!mascot.isFolderPose())
            toast(if (mascot.isFolderPose()) "Inspection pose preview." else "Idle pose.")
            true
        }
        root.addView(mascot, mascotParams)
        setContentView(root)
        if (!getPreferences(MODE_PRIVATE).getBoolean("mascot_tutorial_seen", false)) {
            handler.postDelayed({ if (!isFinishing) showMascotTutorial() }, 700)
        }
    }

    private fun scheduleInventoryRefresh() {
        val constraints = Constraints.Builder()
            .setRequiresBatteryNotLow(true)
            .build()
        val request = PeriodicWorkRequestBuilder<AppInventoryRefreshWorker>(12, TimeUnit.HOURS)
            .setConstraints(constraints)
            .build()
        WorkManager.getInstance(this).enqueueUniquePeriodicWork(
            "crimson-app-inventory",
            ExistingPeriodicWorkPolicy.KEEP,
            request
        )
    }

    private fun showLoadingScreen() {
        val root = FrameLayout(this).apply {
            setBackgroundColor(bg)
            setPadding(dp(24), dp(24), dp(24), dp(24))
        }
        val stack = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
        }
        stack.addView(ImageView(this).apply {
            setImageResource(R.drawable.crimson_mark)
            layoutParams = LinearLayout.LayoutParams(dp(64), dp(64))
        })
        stack.addView(space(16))
        stack.addView(label("CRIMSON SYSTEM BOOST", 21, textMain, true))
        stack.addView(label("INITIALIZING DEVICE INSPECTION", 10, accent, true))
        stack.addView(space(12))
        stack.addView(label("Preparing isolated diagnostics and app inventory…", 13, textMuted))
        root.addView(stack, FrameLayout.LayoutParams(-1, -1))
        val mascot = CrimsonMascotView(this).apply {
            setFolderPose(true)
            contentDescription = "Crimson mascot holding an inspection folder"
        }
        root.addView(mascot, FrameLayout.LayoutParams(dp(112), dp(154), Gravity.BOTTOM or Gravity.CENTER_HORIZONTAL).apply {
            bottomMargin = dp(18)
        })
        setContentView(root)
    }

    private fun showMascotTutorial() {
        AlertDialog.Builder(this)
            .setTitle("Meet your Crimson Guardian")
            .setMessage("DRAG  — Move the guardian anywhere inside the app.\n\nRELEASE  — It settles with a restrained landing motion.\n\nTAP  — Reopen this guide.\n\nHOLD  — Preview the inspection-folder pose.\n\nThe guardian uses a compact chibi silhouette with a serious masked design. File scanning is not connected in this build.")
            .setPositiveButton("Got it") { _, _ ->
                getPreferences(MODE_PRIVATE).edit().putBoolean("mascot_tutorial_seen", true).apply()
            }
            .setNegativeButton("Later", null)
            .show()
    }

    private fun hardwareSummary(): String {
        val am = getSystemService(ACTIVITY_SERVICE) as ActivityManager
        val memory = ActivityManager.MemoryInfo()
        am.getMemoryInfo(memory)
        val stat = StatFs(filesDir.absolutePath)
        val totalStorage = stat.totalBytes / (1024L * 1024L * 1024L)
        val freeStorage = stat.availableBytes / (1024L * 1024L * 1024L)
        val batteryManager = getSystemService(BATTERY_SERVICE) as BatteryManager
        val battery = batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
        val batteryIntent = registerReceiver(null, android.content.IntentFilter(Intent.ACTION_BATTERY_CHANGED))
        val tempTenths = batteryIntent?.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, -1) ?: -1
        val batteryTemp = if (tempTenths >= 0) "${"%.1f".format(tempTenths / 10f)} °C" else "N/A"
        val charging = when (batteryIntent?.getIntExtra(BatteryManager.EXTRA_STATUS, -1)) {
            BatteryManager.BATTERY_STATUS_CHARGING, BatteryManager.BATTERY_STATUS_FULL -> "Charging"
            BatteryManager.BATTERY_STATUS_DISCHARGING -> "On battery"
            BatteryManager.BATTERY_STATUS_NOT_CHARGING -> "Not charging"
            else -> "Unknown"
        }
        val refresh = if (Build.VERSION.SDK_INT >= 30) display?.refreshRate ?: 0f else @Suppress("DEPRECATION") windowManager.defaultDisplay.refreshRate
        val supportedRefreshRates = if (Build.VERSION.SDK_INT >= 23) {
            (display?.supportedModes ?: emptyArray()).map { it.refreshRate.toInt() }.distinct().sorted().joinToString(", ")
        } else ""
        val usedRam = (memory.totalMem - memory.availMem) / (1024L * 1024L)
        val totalRam = memory.totalMem / (1024L * 1024L)
        return "RAM       $usedRam / $totalRam MB\n" +
            "STORAGE   $freeStorage / $totalStorage GB free\n" +
            "BATTERY   ${if (battery >= 0) "$battery%" else "N/A"} • $charging\n" +
            "TEMP      $batteryTemp\n" +
            "DISPLAY   ${"%.1f".format(refresh)} Hz\n" +
            (if (supportedRefreshRates.isNotBlank()) "MODES     ${supportedRefreshRates} Hz\n" else "") +
            "DEVICE    ${Build.MANUFACTURER} ${Build.MODEL}\n" +
            "ANDROID   ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})"
    }

    private fun chooseGame(onPick: (String, String) -> Unit) {
        val intent = Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER)
        val activities: List<ResolveInfo> = if (Build.VERSION.SDK_INT >= 33) {
            packageManager.queryIntentActivities(intent, PackageManager.ResolveInfoFlags.of(0))
        } else {
            @Suppress("DEPRECATION")
            packageManager.queryIntentActivities(intent, 0)
        }
        val choices = activities
            .filter { it.activityInfo.packageName != packageName }
            .distinctBy { it.activityInfo.packageName }
            .sortedBy { it.loadLabel(packageManager).toString().lowercase() }
        if (choices.isEmpty()) {
            toast("No launchable apps found.")
            return
        }
        val labels = choices.map { it.loadLabel(packageManager).toString() }.toTypedArray()
        AlertDialog.Builder(this)
            .setTitle("Select an app")
            .setItems(labels) { _, index ->
                onPick(choices[index].activityInfo.packageName, labels[index])
            }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun confirm(title: String, message: String, positive: String, action: () -> Unit) {
        AlertDialog.Builder(this)
            .setTitle(title)
            .setMessage(message)
            .setNegativeButton("Cancel", null)
            .setPositiveButton(positive) { _, _ -> action() }
            .show()
    }

    private fun connectShizuku() {
        if (!Shizuku.pingBinder()) {
            AlertDialog.Builder(this)
                .setTitle("Shizuku is not running")
                .setMessage("Install Shizuku, start its service using Wireless debugging or root, then return here and connect.")
                .setNegativeButton("Close", null)
                .setPositiveButton("Download Shizuku") { _, _ ->
                    safeOpen(Intent(Intent.ACTION_VIEW, Uri.parse("https://shizuku.rikka.app/download/")))
                }
                .show()
            updateShizukuStatus()
            return
        }
        if (Shizuku.isPreV11()) {
            toast("This build requires Shizuku API 11 or newer.")
            return
        }
        when (Shizuku.checkSelfPermission()) {
            PackageManager.PERMISSION_GRANTED -> bindAdvancedService()
            else -> Shizuku.requestPermission(SHIZUKU_REQUEST_CODE)
        }
    }

    private fun openShizukuManager() {
        val launch = packageManager.getLaunchIntentForPackage("moe.shizuku.privileged.api")
        if (launch != null) safeOpen(launch)
        else safeOpen(Intent(Intent.ACTION_VIEW, Uri.parse("https://shizuku.rikka.app/download/")))
    }

    private fun runAdvanced(label: String, action: (IAdvancedService) -> String) {
        if (!Shizuku.pingBinder()) {
            toast("Start Shizuku first.")
            updateShizukuStatus()
            return
        }
        if (Shizuku.checkSelfPermission() != PackageManager.PERMISSION_GRANTED) {
            pendingAdvancedAction = action
            Shizuku.requestPermission(SHIZUKU_REQUEST_CODE)
            return
        }
        pendingAdvancedAction = action
        if (advancedService != null) {
            val service = advancedService
            pendingAdvancedAction = null
            if (service != null) executeAdvanced(action, service)
        } else {
            bindAdvancedService()
            toast("Connecting to Shizuku…")
        }
    }

    private fun bindAdvancedService() {
        if (!Shizuku.pingBinder()) {
            toast("Shizuku connection was lost.")
            return
        }
        if (advancedService != null) {
            val action = pendingAdvancedAction
            pendingAdvancedAction = null
            if (action != null) executeAdvanced(action, advancedService!!)
            updateShizukuStatus()
            return
        }
        try {
            Shizuku.bindUserService(userServiceArgs, serviceConnection)
            serviceBound = true
        } catch (error: Exception) {
            pendingAdvancedAction = null
            toast("Could not start advanced service: ${error.message ?: "unknown error"}")
        }
    }

    private fun executeAdvanced(action: (IAdvancedService) -> String, service: IAdvancedService) {
        executor.execute {
            val result = try {
                action(service)
            } catch (error: Exception) {
                "Operation failed: ${error.message ?: error.javaClass.simpleName}"
            }
            runOnUiThread {
                AlertDialog.Builder(this)
                    .setTitle("System operation result")
                    .setMessage(result)
                    .setPositiveButton("Close", null)
                    .show()
            }
        }
    }

    private fun updateShizukuStatus() {
        if (!::shizukuStatus.isInitialized) return
        val state = try {
            when {
                !Shizuku.pingBinder() -> "SHIZUKU STATUS: NOT RUNNING"
                Shizuku.checkSelfPermission() != PackageManager.PERMISSION_GRANTED -> "SHIZUKU STATUS: PERMISSION REQUIRED"
                advancedService != null -> "SHIZUKU STATUS: CONNECTED"
                else -> "SHIZUKU STATUS: AVAILABLE"
            }
        } catch (_: Exception) {
            "SHIZUKU STATUS: UNAVAILABLE"
        }
        shizukuStatus.text = state
        shizukuStatus.setTextColor(if (state.endsWith("CONNECTED")) success else warning)
    }

    private fun sendMessage() {
        val endpoint = endpointInput.text.toString().trim()
        val message = messageInput.text.toString().trim()
        if (!endpoint.startsWith("https://") || !endpoint.endsWith("/api/chat")) {
            toast("Save a valid HTTPS backend first.")
            return
        }
        if (message.isBlank()) {
            toast("Enter a message first.")
            return
        }
        messageInput.isEnabled = false
        chatLog.append("\n\nYou: $message\nCrimson: Processing…")
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
                    val body = (if (code in 200..299) connection.inputStream else connection.errorStream)
                        ?.bufferedReader()?.use { it.readText() }.orEmpty()
                    if (code !in 200..299) {
                        val detail = try { JSONObject(body).optString("error", "HTTP $code") } catch (_: Exception) { "HTTP $code" }
                        "Request failed: $detail"
                    } else JSONObject(body).optString("reply").ifBlank { "The server returned an empty response." }
                } finally {
                    connection.disconnect()
                }
            } catch (_: Exception) {
                "Could not reach the backend. Check the connection and endpoint."
            }
            runOnUiThread {
                chatLog.append("\n$answer")
                messageInput.isEnabled = true
            }
        }
    }

    private fun cardContainer() = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(dp(16), dp(15), dp(16), dp(15))
        background = roundedBackground(surface, Color.rgb(62, 43, 58), 18)
        elevation = dp(3).toFloat()
        layoutParams = LinearLayout.LayoutParams(-1, -2).apply {
            setMargins(0, dp(6), 0, dp(6))
        }
    }

    private fun roundedBackground(fill: Int, stroke: Int, radiusDp: Int) =
        GradientDrawable().apply {
            setColor(fill)
            cornerRadius = dp(radiusDp).toFloat()
            if (stroke != Color.TRANSPARENT) setStroke(dp(1), stroke)
        }

    private fun accentBackground() = GradientDrawable(
        GradientDrawable.Orientation.LEFT_RIGHT,
        intArrayOf(Color.rgb(255, 76, 94), Color.rgb(190, 25, 61))
    ).apply { cornerRadius = dp(14).toFloat() }

    private fun sectionTitle(value: String) = label(value, 12, textMuted, true).apply {
        letterSpacing = 0.08f
        setPadding(0, dp(8), 0, dp(3))
    }

    private fun label(value: String, size: Int, color: Int, bold: Boolean = false) =
        TextView(this).apply {
            text = value
            textSize = size.toFloat()
            setTextColor(color)
            if (bold) typeface = Typeface.create("sans-serif", Typeface.BOLD)
            setPadding(0, dp(4), 0, dp(4))
        }

    private fun primaryAction(value: String, action: () -> Unit) = Button(this).apply {
        text = value
        textSize = 14f
        isAllCaps = false
        typeface = Typeface.create("sans-serif", Typeface.BOLD)
        setTextColor(Color.WHITE)
        background = accentBackground()
        minHeight = dp(48)
        setPadding(dp(16), dp(10), dp(16), dp(10))
        stateListAnimator = null
        setOnClickListener { action() }
    }

    private fun secondaryAction(value: String, action: () -> Unit) = Button(this).apply {
        text = value
        textSize = 13f
        isAllCaps = false
        typeface = Typeface.create("sans-serif-medium", Typeface.NORMAL)
        setTextColor(textMain)
        background = roundedBackground(surfaceAlt, Color.rgb(77, 59, 77), 13)
        minHeight = dp(46)
        setPadding(dp(14), dp(9), dp(14), dp(9))
        stateListAnimator = null
        setOnClickListener { action() }
    }

    private fun space(height: Int) = View(this).apply {
        layoutParams = LinearLayout.LayoutParams(1, dp(height))
    }

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).toInt()
    private fun safeOpen(intent: Intent) {
        try { startActivity(intent) } catch (_: Exception) { toast("Unable to open this destination.") }
    }
    private fun toast(message: String) = Toast.makeText(this, message, Toast.LENGTH_SHORT).show()

    companion object {
        private const val SHIZUKU_REQUEST_CODE = 4107
    }
}
