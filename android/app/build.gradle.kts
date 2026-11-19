plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "md.comic.phone"
    compileSdk = 36

    defaultConfig {
        applicationId = "md.comic.phone"
        // Android 7.0. Mai jos nu are rost: AudioRecord si UDP merg si pe mai
        // vechi, dar telefoanele de dinainte de 2016 nu mai primesc actualizari.
        minSdk = 24
        // Android 16. Google nu primeste aplicatii noi care tintesc mai jos,
        // din 31 august 2026 (vezi docs/LANSARE.md). `targetSdk` nu e o
        // formalitate: Android isi schimba purtarea dupa el, nu dupa ce
        // telefon ai. Ce ne-ar fi atins din Android 16 si de ce nu ne atinge:
        //   - marginile ecranului se iau cu sila: le ceream oricum singuri,
        //     cu safeDrawingPadding (vezi ui/ComicApp.kt);
        //   - butonul inapoi cu previziune se aprinde singur: nu il prindem
        //     nicaieri, deci nu avem ce strica;
        //   - paginile de 16 KB pentru cod nativ: nu avem cod nativ;
        //   - orientarea fortata se ignora pe ecrane mari: nu o fortam.
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
        // Ne trebuie pentru BuildConfig.DEBUG, care decide daca se vede
        // comutatorul de proba pentru Pro (vezi ui/SettingsScreen.kt).
        // Verificarea e la compilare, deci randul acela nu ajunge deloc in
        // aplicatia de lansare, nu doar ascuns in ea.
        buildConfig = true
    }

    // Sursele stau in kotlin/, nu in java/.
    sourceSets {
        getByName("main") { kotlin.srcDir("src/main/kotlin") }
        getByName("test") { kotlin.srcDir("src/test/kotlin") }
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    // Corutinele vin si indirect, prin lifecycle; le cerem pe fata ca sa nu
    // ajungem la o versiune aleasa de altcineva.
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.activity.compose)

    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    debugImplementation(libs.androidx.compose.ui.tooling)

    // Protocolul si descoperirea sunt Kotlin curat, fara Android, deci se
    // testeaza pe JVM: `./gradlew test` nu are nevoie de telefon.
    testImplementation(libs.junit)
}
