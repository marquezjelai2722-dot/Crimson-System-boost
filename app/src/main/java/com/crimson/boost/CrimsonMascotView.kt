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

/**
 * Lightweight, original pixel-block mascot drawn at runtime.
 * Dragging changes its pose; releasing plays a short fall-and-land reaction.
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
                    walkingPhase += 0.45f
                    invalidate()
                }
                return true
            }
            MotionEvent.ACTION_UP -> {
                parentView?.requestDisallowInterceptTouchEvent(false)
                if (moved) {
                    moved = false
                    playDropReaction()
                } else performClick()
                return true
            }
            MotionEvent.ACTION_CANCEL -> {
                parentView?.requestDisallowInterceptTouchEvent(false)
                if (moved) {
                    moved = false
                    playDropReaction()
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
        animate().cancel()
        animate().rotation(11f).translationY(22f).setDuration(170).withEndAction {
            animate().rotation(0f).translationY(0f).setDuration(230).withEndAction {
                animate().translationY(0f).setDuration(120).start()
            }.start()
        }.start()
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val save = canvas.save()
        val scale = minOf(width / 112f, height / 154f)
        canvas.translate((width - 112f * scale) / 2f, (height - 154f * scale) / 2f)
        canvas.scale(scale, scale)

        val black = Color.rgb(20, 20, 23)
        val dark = Color.rgb(39, 39, 43)
        val mid = Color.rgb(65, 65, 70)
        val light = Color.rgb(83, 83, 89)
        val orange = Color.rgb(255, 105, 18)
        val glow = Color.rgb(255, 54, 70)

        // Soft ground shadow and hood silhouette.
        block(canvas, 22f, 143f, 70f, 7f, Color.argb(95, 0, 0, 0))
        block(canvas, 30f, 5f, 54f, 55f, black)
        block(canvas, 25f, 13f, 9f, 39f, dark)
        block(canvas, 80f, 12f, 9f, 42f, dark)
        block(canvas, 36f, 0f, 42f, 10f, mid)
        block(canvas, 35f, 9f, 44f, 39f, dark)
        block(canvas, 39f, 15f, 38f, 29f, black)

        // Distinctive orange pixel mask marking.
        block(canvas, 58f, 15f, 19f, 8f, orange)
        block(canvas, 49f, 23f, 28f, 8f, orange)
        block(canvas, 56f, 31f, 15f, 8f, orange)
        block(canvas, 62f, 39f, 8f, 7f, orange)
        block(canvas, 77f, 23f, 5f, 8f, orange)

        // Hooded torso and angular armor.
        val stride = if (moved) kotlin.math.sin(walkingPhase.toDouble()).toFloat() * 5f else 0f
        block(canvas, 28f, 56f, 58f, 48f, black)
        block(canvas, 21f, 62f, 14f, 39f, dark)
        block(canvas, 79f, 62f, 14f, 39f, dark)
        block(canvas, 34f, 59f, 46f, 35f, mid)
        block(canvas, 39f, 68f, 35f, 29f, dark)
        block(canvas, 29f + stride, 99f, 24f, 45f, black)
        block(canvas, 59f - stride, 99f, 24f, 45f, black)
        block(canvas, 34f + stride, 104f, 12f, 34f, dark)
        block(canvas, 65f - stride, 104f, 12f, 34f, dark)
        block(canvas, 28f + stride, 139f, 27f, 8f, black)
        block(canvas, 57f - stride, 139f, 29f, 8f, black)

        // Chunky sleeves and crossed forearms.
        block(canvas, 15f, 63f, 22f, 14f, dark)
        block(canvas, 75f, 63f, 22f, 14f, dark)
        block(canvas, 23f, 73f, 62f, 12f, light)
        block(canvas, 33f, 83f, 51f, 10f, dark)
        block(canvas, 28f, 76f, 15f, 13f, mid)
        block(canvas, 70f, 76f, 15f, 13f, mid)

        if (folderPose) {
            block(canvas, 40f, 72f, 36f, 23f, Color.rgb(196, 104, 18))
            block(canvas, 43f, 68f, 16f, 7f, orange)
            block(canvas, 44f, 77f, 27f, 14f, Color.rgb(255, 145, 27))
            block(canvas, 54f, 81f, 10f, 6f, Color.rgb(255, 205, 100))
        } else {
            // Red chest sigil, visible in the crossed-arms idle pose.
            block(canvas, 50f, 72f, 13f, 10f, glow)
            block(canvas, 54f, 75f, 6f, 4f, Color.WHITE)
        }

        // Pixel edge highlights.
        block(canvas, 29f, 18f, 4f, 28f, light)
        block(canvas, 82f, 18f, 3f, 27f, mid)
        block(canvas, 19f, 66f, 5f, 25f, mid)
        block(canvas, 91f, 67f, 4f, 24f, mid)
        canvas.restoreToCount(save)
    }

    /** Switch to the folder-holding pose only when a real scan workflow is connected. */
    fun setFolderPose(enabled: Boolean) {
        folderPose = enabled
        invalidate()
    }

    private fun block(canvas: Canvas, left: Float, top: Float, width: Float, height: Float, color: Int) {
        paint.color = color
        paint.style = Paint.Style.FILL
        canvas.drawRect(RectF(left, top, left + width, top + height), paint)
    }
}
