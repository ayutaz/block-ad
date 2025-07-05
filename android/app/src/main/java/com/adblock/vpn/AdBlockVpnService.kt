package com.adblock.vpn

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import androidx.core.app.NotificationCompat
import com.adblock.AdBlockEngine
import com.adblock.MainActivity
import com.adblock.R
import com.adblock.Statistics
import java.io.FileInputStream
import java.io.FileOutputStream
import java.net.InetAddress
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.concurrent.thread

/**
 * VPN Service for system-wide ad blocking
 */
class AdBlockVpnService : VpnService() {
    
    companion object {
        private const val CHANNEL_ID = "AdBlockVPN"
        private const val NOTIFICATION_ID = 1
        private const val MTU_SIZE = 1500
    }
    
    private var vpnInterface: ParcelFileDescriptor? = null
    private val isRunning = AtomicBoolean(false)
    private lateinit var engine: AdBlockEngine
    private var filterRules: String = ""
    
    private var readThread: Thread? = null
    private var writeThread: Thread? = null
    
    override fun onCreate() {
        super.onCreate()
        engine = AdBlockEngine()
        createNotificationChannel()
    }
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (isRunning.get()) {
            return START_STICKY
        }
        
        // Start VPN
        startVpn()
        
        return START_STICKY
    }
    
    override fun onDestroy() {
        stop()
        engine.destroy()
        super.onDestroy()
    }
    
    fun isRunning(): Boolean = isRunning.get()
    
    fun stop() {
        isRunning.set(false)
        
        readThread?.interrupt()
        writeThread?.interrupt()
        
        vpnInterface?.close()
        vpnInterface = null
        
        stopForeground(true)
    }
    
    fun setFilterRules(rules: String) {
        filterRules = rules
        engine.loadFilterList(rules)
    }
    
    fun shouldBlockPacket(packet: NetworkPacket): Boolean {
        val url = "https://${packet.host}:${packet.port}"
        return engine.shouldBlock(url)
    }
    
    fun getStatistics(): Statistics = engine.getStatistics()
    
    private fun startVpn() {
        val builder = Builder()
        
        // Configure VPN
        builder.setSession("AdBlock VPN")
            .setMtu(MTU_SIZE)
            .addAddress("10.0.0.2", 24)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("8.8.8.8")
            .addDnsServer("8.8.4.4")
        
        // Establish VPN
        vpnInterface = builder.establish() ?: return
        
        isRunning.set(true)
        
        // Start packet processing threads
        startPacketProcessing()
        
        // Show notification
        startForeground(NOTIFICATION_ID, createNotification())
    }
    
    private fun startPacketProcessing() {
        val vpnInput = FileInputStream(vpnInterface!!.fileDescriptor)
        val vpnOutput = FileOutputStream(vpnInterface!!.fileDescriptor)
        
        // Read packets from VPN interface
        readThread = thread {
            val buffer = ByteBuffer.allocate(MTU_SIZE)
            
            while (isRunning.get()) {
                try {
                    buffer.clear()
                    val length = vpnInput.channel.read(buffer)
                    if (length > 0) {
                        buffer.flip()
                        processPacket(buffer, vpnOutput)
                    }
                } catch (e: Exception) {
                    if (isRunning.get()) {
                        e.printStackTrace()
                    }
                }
            }
        }
    }
    
    private fun processPacket(packet: ByteBuffer, output: FileOutputStream) {
        // Simple packet inspection (in real implementation, parse IP/TCP/UDP headers)
        // For now, just forward all packets
        
        try {
            output.channel.write(packet)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }
    
    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "AdBlock VPN Service",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Shows when AdBlock VPN is active"
            }
            
            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager.createNotificationChannel(channel)
        }
    }
    
    private fun createNotification(): Notification {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("AdBlock VPN")
            .setContentText("Ad blocking is active")
            .setSmallIcon(android.R.drawable.ic_lock_lock)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }
}

/**
 * Network packet data class
 */
data class NetworkPacket(
    val host: String,
    val port: Int,
    val size: Int
)