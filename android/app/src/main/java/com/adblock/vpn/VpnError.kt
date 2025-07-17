package com.adblock.vpn

/**
 * VPN service error types
 */
sealed class VpnError : Exception() {
    data class PacketParsingError(override val message: String) : VpnError()
    data class NetworkError(override val message: String) : VpnError()
    data class EngineError(override val message: String) : VpnError()
    data class ConfigurationError(override val message: String) : VpnError()
    data class PermissionError(override val message: String) : VpnError()
}

/**
 * Result type for VPN operations
 */
sealed class VpnResult<out T> {
    data class Success<T>(val data: T) : VpnResult<T>()
    data class Failure(val error: VpnError) : VpnResult<Nothing>()
    
    inline fun <R> map(transform: (T) -> R): VpnResult<R> = when (this) {
        is Success -> Success(transform(data))
        is Failure -> this
    }
    
    inline fun <R> flatMap(transform: (T) -> VpnResult<R>): VpnResult<R> = when (this) {
        is Success -> transform(data)
        is Failure -> this
    }
    
    fun getOrNull(): T? = when (this) {
        is Success -> data
        is Failure -> null
    }
    
    fun getOrThrow(): T = when (this) {
        is Success -> data
        is Failure -> throw error
    }
}

/**
 * Extension functions for Result handling
 */
inline fun <T> VpnResult<T>.onSuccess(action: (T) -> Unit): VpnResult<T> {
    if (this is VpnResult.Success) {
        action(data)
    }
    return this
}

inline fun <T> VpnResult<T>.onFailure(action: (VpnError) -> Unit): VpnResult<T> {
    if (this is VpnResult.Failure) {
        action(error)
    }
    return this
}

/**
 * Helper to create results
 */
fun <T> vpnSuccess(data: T): VpnResult<T> = VpnResult.Success(data)
fun <T> vpnFailure(error: VpnError): VpnResult<T> = VpnResult.Failure(error)

/**
 * Try-catch wrapper for VPN operations
 */
inline fun <T> vpnTry(block: () -> T): VpnResult<T> {
    return try {
        VpnResult.Success(block())
    } catch (e: VpnError) {
        VpnResult.Failure(e)
    } catch (e: Exception) {
        VpnResult.Failure(VpnError.NetworkError(e.message ?: "Unknown error"))
    }
}