// Pluginurile se declara aici si se aplica in app/build.gradle.kts.
// `apply false` inseamna "incarca-l, dar nu-l porni in proiectul radacina".
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
}
