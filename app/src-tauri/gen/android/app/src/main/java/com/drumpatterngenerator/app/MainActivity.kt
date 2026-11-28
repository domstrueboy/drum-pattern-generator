package com.drumpatterngenerator.app

import android.content.pm.ActivityInfo
import android.os.Bundle
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  // The screen never turns by itself: the arrow button steps through the
  // four orientations, a quarter turn each press.
  private val orientations = intArrayOf(
    ActivityInfo.SCREEN_ORIENTATION_PORTRAIT,
    ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE,
    ActivityInfo.SCREEN_ORIENTATION_REVERSE_PORTRAIT,
    ActivityInfo.SCREEN_ORIENTATION_REVERSE_LANDSCAPE,
  )
  private var orientation = 0

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    requestedOrientation = orientations[orientation]
    super.onCreate(savedInstanceState)
  }

  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(Orientation(), "DpgOrientation")
  }

  /** What the page may call: `window.DpgOrientation.next()`. */
  inner class Orientation {
    @JavascriptInterface
    fun next() {
      runOnUiThread {
        orientation = (orientation + 1) % orientations.size
        requestedOrientation = orientations[orientation]
      }
    }
  }
}
