package com.crimson.boost.core.inventory

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class InstalledAppInventoryTest {
    @Test
    fun largeInstallFootprintIsNotTreatedAsMalware() {
        val app = InstalledAppRecord(
            packageName = "example.large",
            displayName = "Large Example",
            versionName = "1.0",
            origin = AppOrigin.USER,
            apkSizeBytes = 150L * 1024L * 1024L,
            footprint = InstallFootprint.LARGE
        )

        assertEquals(SecurityVerdict.NOT_SCANNED, app.verdict)
        assertEquals(InstallFootprint.LARGE, app.footprint)
    }

    @Test
    fun inventoryCountsSystemAndUserAppsSeparately() {
        val report = InventoryReport(
            apps = listOf(
                record("system.app", AppOrigin.SYSTEM),
                record("user.app", AppOrigin.USER),
                record("user.large", AppOrigin.USER, InstallFootprint.LARGE)
            ),
            skippedPackages = 1,
            generatedAtMillis = 100L
        )

        assertEquals(1, report.systemApps)
        assertEquals(2, report.userApps)
        assertEquals(1, report.largeApks)
        assertEquals(1, report.skippedPackages)
        assertFalse(report.apps.any { it.verdict == SecurityVerdict.CONFIRMED_THREAT })
    }

    private fun record(
        name: String,
        origin: AppOrigin,
        footprint: InstallFootprint = InstallFootprint.SMALL
    ) = InstalledAppRecord(
        packageName = name,
        displayName = name,
        versionName = "1",
        origin = origin,
        apkSizeBytes = 1024L,
        footprint = footprint
    )
}
