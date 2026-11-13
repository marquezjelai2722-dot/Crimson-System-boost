package com.crimson.boost

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.RectF
import android.view.MotionEvent
import android.view.View
import android.view.ViewGroup
import kotlin.math.abs
import kotlin.math.sin

/**
 * Crimson Guardian: restrained chibi proportions, pixel-mask identity,
 * deliberate movement and a quiet landing response.
 */
class CrimsonMascotView(context: Context) : View(context) {
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private var downRawX = 0f
    private var downRawY = 0f
    private var startX = 0f
    private var startY = 0f
    private var moved = false
    private var walkingPhase = 0f
    private var folderPose = false
    private var dropAnimatorRunning = false

    init {
        isClickable = true
        isFocusable = true
        setLayerType(LAYER_TYPE_SOFTWARE, null)
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        val parentView = parent as? ViewGroup
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                downRawX = event.rawX
                downRawY = event.rawY
                startX = x
                startY = y
                moved = false
                parentView?.requestDisallowInterceptTouchEvent(true)
                return true
            }
            MotionEvent.ACTION_MOVE -> {
                val dx = event.rawX - downRawX
                val dy = event.rawY - downRawY
                if (abs(dx) + abs(dy) > 8f) moved = true
                if (moved && parentView != null) {
                    x = (startX + dx).coerceIn(0f, (parentView.width - width).coerceAtLeast(0).toFloat())
                    y = (startY + dy).coerceIn(0f, (parentView.height - height).coerceAtLeast(0).toFloat())
                    walkingPhase += 0.32f
                    rotation = sin(walkingPhase.toDouble()).toFloat() * 1.4f
                    invalidate()
                }
                return true
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                parentView?.requestDisallowInterceptTouchEvent(false)
                if (moved) {
                    moved = false
                    playDropReaction()
                } else if (event.actionMasked == MotionEvent.ACTION_UP) {
                    performClick()
                }
                return true
            }
        }
        return true
    }

    override fun performClick(): Boolean {
        super.performClick()
        return true
    }

    private fun playDropReaction() {
        if (dropAnimatorRunning) return
        dropAnimatorRunning = true
        animate().cancel()
        animate()
            .rotation(8f)
            .translationY(18f)
            .setDuration(150)
            .withEndAction {
                animate()
                    .rotation(-2f)
                    .translationY(-5f)
                    .setDuration(180)
                    .withEndAction {
                        animate()
                            .rotation(0f)
                            .translationY(0f)
                            .setDuration(160)
                            .withEndAction { dropAnimatorRunning = false }
                            .start()
                    }
                    .start()
            }
            .start()
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val save = canvas.save()
        val scale = minOf(width / 112f, height / 154f)
        canvas.translate((width - 112f * scale) / 2f, (height - 154f * scale) / 2f)
        canvas.scale(scale, scale)

        val hood = Color.rgb(25, 25, 29)
        val black = Color.rgb(13, 13, 16)
        val dark = Color.rgb(37, 37, 42)
        val mid = Color.rgb(62, 62, 69)
        val edge = Color.rgb(83, 83, 91)
        val orange = Color.rgb(255, 111, 24)
        val red = Color.rgb(255, 54, 70)

        // Ground shadow.
        block(canvas, 29f, 146f, 56f, 5f, Color.argb(105, 0, 0, 0))

        // Oversized hood/head: deliberately chibi, with a narrow dark faceplate.
        block(canvas, 19f, 4f, 74f, 68f, black)
        block(canvas, 24f, 0f, 64f, 10f, mid)
        block(canvas, 15f, 15f, 12f, 42f, dark)
        block(canvas, 85f, 13f, 12f, 44f, dark)
        block(canvas, 23f, 10f, 66f, 53f, hood)
        block(canvas, 31f, 17f, 51f, 40f, black)
        block(canvas, 27f, 56f, 9f, 10f, dark)
        block(canvas, 77f, 55f, 9f, 11f, dark)

        // Asymmetric orange mask mark, echoing the original reference.
        block(canvas, 58f, 19f, 19f, 8f, orange)
        block(canvas, 43f, 27f, 34f, 9f, orange)
        block(canvas, 53f, 36f, 20f, 9f, orange)
        block(canvas, 60f, 45f, 10f, 8f, orange)
        block(canvas, 77f, 27f, 7f, 9f, orange)
        block(canvas, 28f, 21f, 4f, 29f, edge)
        block(canvas, 82f, 20f, 4f, 30f, mid)

        // Compact armored body.
        block(canvas, 31f, 66f, 50f, 43f, black)
        block(canvas, 25f, 72f, 12f, 31f, dark)
        block(canvas, 75f, 72f, 12f, 31f, dark)
        block(canvas, 35f, 70f, 42f, 31f, mid)
        block(canvas, 40f, 79f, 32f, 21f, dark)
        block(canvas, 34f, 102f, 19f, 39f, black)
        block(canvas, 59f, 102f, 19f, 39f, black)

        // Small legs shift subtly during drag; no exaggerated cartoon gait.
        val stride = if (moved) sin(walkingPhase.toDouble()).toFloat() * 4f else 0f
        block(canvas, 36f + stride, 108f, 10f, 30f, dark)
        block(canvas, 62f - stride, 108f, 10f, 30f, dark)
        block(canvas, 29f + stride, 137f, 23f, 8f, black)
        block(canvas, 58f - stride, 137f, 25f, 8f, black)

        // Folded arms across the chest.
        block(canvas, 22f, 76f, 22f, 13f, dark)
        block(canvas, 69f, 76f, 22f, 13f, dark)
        block(canvas, 31f, 81f, 51f, 11f, edge)
        block(canvas, 39f, 89f, 38f, 9f, dark)
        block(canvas, 34f, 82f, 15f, 12f, mid)
        block(canvas, 67f, 82f, 15f, 12f, mid)

        if (folderPose) {
            // Folder sits against the chest; one forearm supports it while the
            // opposite arm remains visibly folded across the body.
            block(canvas, 45f, 78f, 31f, 21f, Color.rgb(150, 77, 24))
            block(canvas, 49f, 74f, 14f, 6f, orange)
            block(canvas, 48f, 82f, 25f, 13f, Color.rgb(220, 112, 27))
            block(canvas, 52f, 85f, 16f, 2f, Color.rgb(255, 176, 82))
            block(canvas, 30f, 86f, 28f, 8f, dark)
        } else {
            block(canvas, 51f, 83f, 12f, 8f, red)
            block(canvas, 55f, 85f, 5f, 3f, Color.rgb(245, 230, 230))
        }

        canvas.restoreToCount(save)
    }

    /** Preview pose; wire this to a real scan state only after scanning exists. */
    fun setFolderPose(enabled: Boolean) {
        folderPose = enabled
        invalidate()
    }

    fun isFolderPose(): Boolean = folderPose

    private fun block(canvas: Canvas, left: Float, top: Float, width: Float, height: Float, color: Int) {
        paint.color = color
        paint.style = Paint.Style.FILL
        canvas.drawRect(RectF(left, top, left + width, top + height), paint)
    }
}
