package com.adblock

import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.assertIsDisplayed
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4

import org.junit.Test
import org.junit.Rule
import org.junit.runner.RunWith

import org.junit.Assert.*

@RunWith(AndroidJUnit4::class)
class ExampleInstrumentedTest {
    
    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()
    
    @Test
    fun useAppContext() {
        // Context of the app under test.
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertEquals("com.adblock", appContext.packageName)
    }
    
    @Test
    fun mainActivityLaunches() {
        // Check if main activity launches successfully with AdBlock title
        composeTestRule.onNodeWithText("AdBlock").assertIsDisplayed()
    }
    
    @Test
    fun vpnSectionIsDisplayed() {
        // Check if VPN section is displayed
        composeTestRule.onNodeWithText("VPN Status").assertIsDisplayed()
    }
    
    @Test
    fun statisticsSectionIsDisplayed() {
        // Check if statistics section is visible
        composeTestRule.onNodeWithText("Statistics").assertIsDisplayed()
        composeTestRule.onNodeWithText("Ads Blocked").assertIsDisplayed()
    }
    
    @Test
    fun filterListsSectionIsDisplayed() {
        // Check if filter lists section is visible
        composeTestRule.onNodeWithText("Filter Lists").assertIsDisplayed()
    }
}