plugins {
    id("com.android.library")
    kotlin("android")
}

android {
    namespace = "com.personaflux"
    compileSdk = 35

    defaultConfig {
        minSdk = 24
        consumerProguardFiles("consumer-rules.pro")
        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
            arguments("-DPERSONAFLUX_NATIVE_DIR=${System.getenv("PERSONAFLUX_NATIVE_DIR") ?: "${rootDir}/../../target/personaflux-android"}")
        }
    }
}

dependencies {
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test:runner:1.6.2")
}
