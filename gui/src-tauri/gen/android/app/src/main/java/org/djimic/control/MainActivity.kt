package org.djimic.control

import android.content.Intent
import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    DjiUsbManager.start(this)
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    DjiUsbManager.scanNow()
  }

  override fun onResume() {
    super.onResume()
    DjiUsbManager.scanNow()
  }
}
