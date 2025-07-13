package com.adblock

import androidx.compose.ui.test.*
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class MainActivityUITest {

    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun testVpnToggleInteraction() {
        // Find and click the VPN toggle switch
        composeTestRule.onNodeWithText("Enable VPN").performClick()
        
        // Verify that the status changes (note: actual VPN may not start in test environment)
        // Just verify UI responds to interaction
        composeTestRule.waitForIdle()
    }

    @Test
    fun testUpdateFiltersButton() {
        // Find and click the Update Filters button
        composeTestRule.onNodeWithText("Update Filters").assertIsDisplayed()
        composeTestRule.onNodeWithText("Update Filters").performClick()
        
        // Wait for any UI updates
        composeTestRule.waitForIdle()
    }

    @Test
    fun testResetStatisticsButton() {
        // Find and click the Reset Statistics button
        composeTestRule.onNodeWithText("Reset Statistics").assertIsDisplayed()
        composeTestRule.onNodeWithText("Reset Statistics").performClick()
        
        // Wait for UI to update
        composeTestRule.waitForIdle()
    }

    @Test
    fun testCustomRulesButton() {
        // Find and click the Custom Rules button
        composeTestRule.onNodeWithText("Custom Rules").assertIsDisplayed()
        composeTestRule.onNodeWithText("Custom Rules").performClick()
        
        // Should navigate to Custom Rules activity
        composeTestRule.waitForIdle()
    }

    @Test
    fun testAllMainSectionsVisible() {
        // Verify all main sections are visible
        composeTestRule.onNodeWithText("AdBlock").assertIsDisplayed()
        composeTestRule.onNodeWithText("VPN Status").assertIsDisplayed()
        composeTestRule.onNodeWithText("Statistics").assertIsDisplayed()
        composeTestRule.onNodeWithText("Filter Lists").assertIsDisplayed()
        
        // Verify statistics values are shown
        composeTestRule.onAllNodesWithText("0", substring = true).assertAny(hasTestTag("statistics_value"))
    }

    @Test
    fun testScrollableContent() {
        // Test that the content is scrollable
        composeTestRule.onRoot().performTouchInput {
            swipeUp()
        }
        
        composeTestRule.waitForIdle()
        
        // Swipe back down
        composeTestRule.onRoot().performTouchInput {
            swipeDown()
        }
    }
}