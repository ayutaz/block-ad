package com.adblock

/**
 * Statistics data class for tracking ad blocking metrics
 */
data class Statistics(
    val blockedCount: Int,
    val allowedCount: Int,
    val dataSaved: Int
) {
    val blockRate: Double
        get() {
            val total = blockedCount + allowedCount
            return if (total > 0) (blockedCount.toDouble() / total) * 100 else 0.0
        }
}