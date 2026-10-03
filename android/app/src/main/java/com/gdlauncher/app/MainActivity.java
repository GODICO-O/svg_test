package com.gdlauncher.app;

import android.app.NativeActivity;

public class MainActivity extends NativeActivity {
    static {
        System.loadLibrary("gd_launcher_test");
    }
}
