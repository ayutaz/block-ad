package com.adblock.filter

import android.content.Context
import android.content.SharedPreferences

/**
 * Manages custom filter rules
 */
class CustomRulesManager(private val context: Context) {
    
    companion object {
        private const val PREFS_NAME = "custom_filters"
        private const val KEY_CUSTOM_RULES = "custom_rules"
    }
    
    private val prefs: SharedPreferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    
    /**
     * Load custom rules from storage
     */
    fun loadRules(): List<String> {
        val rulesString = prefs.getString(KEY_CUSTOM_RULES, null) ?: return emptyList()
        return rulesString.split("\n").filter { it.isNotBlank() }
    }
    
    /**
     * Save custom rules to storage
     */
    fun saveRules(rules: List<String>) {
        val rulesString = rules.joinToString("\n")
        prefs.edit().putString(KEY_CUSTOM_RULES, rulesString).apply()
    }
    
    /**
     * Add a new rule
     */
    fun addRule(rule: String) {
        val currentRules = loadRules().toMutableList()
        if (!currentRules.contains(rule.trim())) {
            currentRules.add(rule.trim())
            saveRules(currentRules)
        }
    }
    
    /**
     * Remove a rule
     */
    fun removeRule(rule: String) {
        val currentRules = loadRules().toMutableList()
        currentRules.remove(rule)
        saveRules(currentRules)
    }
    
    /**
     * Get custom rules as a filter list string
     */
    fun getAsFilterList(): String {
        val rules = loadRules()
        return rules.joinToString("\n")
    }
    
    /**
     * Clear all custom rules
     */
    fun clearAll() {
        prefs.edit().remove(KEY_CUSTOM_RULES).apply()
    }
}