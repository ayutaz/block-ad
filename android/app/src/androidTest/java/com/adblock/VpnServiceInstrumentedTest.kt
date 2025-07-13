package com.adblock

import android.content.Context
import android.content.SharedPreferences
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.adblock.filter.FilterListManager
import com.adblock.filter.CustomRulesManager

import org.junit.Test
import org.junit.Before
import org.junit.After
import org.junit.runner.RunWith

import org.junit.Assert.*

@RunWith(AndroidJUnit4::class)
class VpnServiceInstrumentedTest {
    
    private lateinit var context: Context
    private lateinit var sharedPrefs: SharedPreferences
    
    @Before
    fun setup() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
        sharedPrefs = context.getSharedPreferences("adblock_prefs", Context.MODE_PRIVATE)
        // Clear preferences before each test
        sharedPrefs.edit().clear().commit()
    }
    
    @After
    fun tearDown() {
        // Clean up after tests
        sharedPrefs.edit().clear().commit()
    }
    
    @Test
    fun testFilterListManagerInitialization() {
        val filterListManager = FilterListManager(context)
        assertNotNull(filterListManager)
        
        // Test that default filter lists are loaded
        val filters = filterListManager.getActiveFilters()
        assertNotNull(filters)
    }
    
    @Test
    fun testCustomRulesManagerPersistence() {
        val customRulesManager = CustomRulesManager(context)
        
        // Add a custom rule
        val testRule = "||test-domain.com^"
        customRulesManager.addRule(testRule)
        
        // Verify rule was added
        val rules = customRulesManager.getRules()
        assertTrue(rules.contains(testRule))
        
        // Create new instance to test persistence
        val newManager = CustomRulesManager(context)
        val persistedRules = newManager.getRules()
        assertTrue(persistedRules.contains(testRule))
    }
    
    @Test
    fun testSharedPreferencesAccess() {
        // Test VPN enabled state persistence
        sharedPrefs.edit().putBoolean("vpn_enabled", true).apply()
        
        val isEnabled = sharedPrefs.getBoolean("vpn_enabled", false)
        assertTrue(isEnabled)
    }
    
    @Test
    fun testAdBlockEngineCreation() {
        // Test that AdBlockEngine can be created in instrumented environment
        val engine = AdBlockEngine()
        assertNotNull(engine)
        
        // Test basic functionality
        val testUrl = "https://doubleclick.net/ads"
        // Engine should block known ad domains
        // Note: This test assumes the engine has default rules loaded
    }
}