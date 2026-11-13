package com.crimson.boost

import android.app.Activity
import android.os.Bundle
import android.os.Build
import android.content.pm.PackageManager
import android.graphics.Color
import android.view.Gravity
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Button
import android.widget.ScrollView

class MainActivity : Activity() {
    private val bg = Color.rgb(12, 16, 25)
    private val panel = Color.rgb(24, 31, 45)
    private val accent = Color.rgb(91, 220, 160)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        render()
    }

    private fun render() {
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(22, 34, 22, 24)
            setBackgroundColor(bg)
        }
        val scroll = ScrollView(this)
        val content = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
        }
        content.addView(label("CRIMSON", 13, accent))
        content.addView(label("SYSTEM BOOST", 28, Color.WHITE))
        content.addView(label("Safe device optimization • v0.1.0", 13, 0xFFADB8C8.toInt()))
        content.addView(space(20))
        content.addView(card("DEVICE PROFILE", deviceSummary()))
        content.addView(card("GAMING MODE", "Prepare a focused gaming profile. Android controls thermal limits; this app will not disable them or promise unsupported FPS gains."))
        content.addView(action("Run device inventory") { showInventory(content) })
        content.addView(action("Open battery settings") {
            startActivity(android.content.Intent(android.provider.Settings.ACTION_BATTERY_SAVER_SETTINGS))
        })
        content.addView(action("Open app settings") {
            startActivity(android.content.Intent(android.provider.Settings.ACTION_MANAGE_APPLICATIONS_SETTINGS))
        })
        content.addView(card("GAMING FIREWALL", "Per-app network blocking requires a user-approved Android VPN service. Firewall controls will be added with an explicit consent screen."))
        content.addView(card("FILE SCANNER", "The scanner can inspect files you select or grant access to. It cannot claim to scan protected system areas without privileged access."))
        content.addView(card("QUARANTINE & RESTORE", "Selected files can be copied into private app storage for isolation. Deletion requires confirmation and is limited by Android permissions."))
        content.addView(card("ADVANCED MODE • SHIZUKU", "Optional integration planned. No commands run until Shizuku is detected, permission is granted, and each action is reviewed."))
        content.addView(label("No blind system-package removal. No thermal bypass. Changes must be explainable and reversible.", 12, 0xFFFFC66D.toInt()))
        scroll.addView(content)
        root.addView(scroll)
        setContentView(root)
    }

    private fun deviceSummary(): String =
        "Manufacturer: ${Build.MANUFACTURER}\nModel: ${Build.MODEL}\nDevice: ${Build.DEVICE}\nAndroid: ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})\nBuild: ${Build.DISPLAY}"

    private fun showInventory(parent: LinearLayout) {
        val pm = packageManager
        val apps = if (Build.VERSION.SDK_INT >= 33) {
            pm.getInstalledApplications(PackageManager.ApplicationInfoFlags.of(0))
        } else {
            @Suppress("DEPRECATION")
            pm.getInstalledApplications(0)
        }
        val userApps = apps.filter { (it.flags and android.content.pm.ApplicationInfo.FLAG_SYSTEM) == 0 }
        parent.addView(card("INVENTORY RESULT", "Visible installed packages: ${apps.size}\nNon-system apps: ${userApps.size}\n\nThis is an inventory only. Package names do not prove an app is safe to remove."))
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
        val t = label(title, 14, accent)
        val b = label(body, 14, Color.WHITE)
        addView(t); addView(b)
        val lp = LinearLayout.LayoutParams(-1, -2)
        lp.setMargins(0, 10, 0, 10)
        layoutParams = lp
    }

    private fun action(text: String, run: () -> Unit) = Button(this).apply {
        this.text = text
        setOnClickListener { run() }
    }

    private fun space(h: Int) = android.view.View(this).apply {
        layoutParams = LinearLayout.LayoutParams(1, h)
    }
}
