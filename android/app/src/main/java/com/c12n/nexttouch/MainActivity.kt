// NextTouch Android — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later

package com.c12n.nexttouch

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.c12n.nexttouch.ui.NextTouchApp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { NextTouchApp() }
    }
}
