package com.adblock.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable

private val LightColorScheme = lightColorScheme(
    primary = androidx.compose.ui.graphics.Color(0xFF1976D2),
    secondary = androidx.compose.ui.graphics.Color(0xFF388E3C),
    tertiary = androidx.compose.ui.graphics.Color(0xFFF57C00)
)

@Composable
fun AdBlockTheme(
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = LightColorScheme,
        content = content
    )
}