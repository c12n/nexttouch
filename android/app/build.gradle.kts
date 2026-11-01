// NextTouch Android app — Copyright (c) 2026 c12n Apps
// SPDX-License-Identifier: GPL-2.0-or-later
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
    id("org.jetbrains.kotlin.plugin.serialization")
}

android {
    namespace = "com.c12n.nexttouch"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.c12n.nexttouch"
        minSdk = 29
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    buildFeatures {
        compose = true
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }

    // 64-bit only, one APK per ABI (user requirement): arm64-v8a + x86_64.
    splits {
        abi {
            reset()
            include("arm64-v8a", "x86_64")
            isUniversalApk = false
        }
    }
}

// The picker catalog is a single source of truth shared with desktop + iOS:
// it is copied from the Rust catalog crate at build time, never duplicated
// in this tree by hand.
val copyCatalog = tasks.register<Copy>("copyCatalog") {
    from(rootProject.file("../crates/nexttouch-catalog/assets/catalog.json"))
    into(layout.projectDirectory.dir("src/main/assets"))
}
tasks.named("preBuild").configure { dependsOn(copyCatalog) }

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2024.12.01")
    implementation(composeBom)
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
    debugImplementation("androidx.compose.ui:ui-tooling")
}
