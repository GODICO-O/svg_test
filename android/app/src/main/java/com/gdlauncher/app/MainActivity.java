package com.gdlauncher.app;

import com.google.androidgamesdk.GameActivity;

public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("gd_launcher_test");
    }
}
