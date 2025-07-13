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
import com.adblock.filter.FilterListManager
import com.adblock.filter.CustomRulesManager
import java.io.FileInputStream
import java.io.FileOutputStream
import java.net.InetAddress
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.Executors
import java.util.concurrent.ScheduledExecutorService
import java.util.concurrent.TimeUnit
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
    private lateinit var filterListManager: FilterListManager
    private lateinit var customRulesManager: CustomRulesManager
    private var filterRules: String = ""
    
    private var readThread: Thread? = null
    private var writeThread: Thread? = null
    private var statsUpdateExecutor: ScheduledExecutorService? = null
    
    override fun onCreate() {
        super.onCreate()
        engine = AdBlockEngine()
        filterListManager = FilterListManager(this)
        customRulesManager = CustomRulesManager(this)
        createNotificationChannel()
        loadAllFilterLists()
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
        
        statsUpdateExecutor?.shutdown()
        statsUpdateExecutor = null
        
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
        
        // Add allowed apps (exclude ourselves to prevent loops)
        try {
            builder.addDisallowedApplication(packageName)
        } catch (e: Exception) {
            e.printStackTrace()
        }
        
        // Establish VPN
        vpnInterface = builder.establish() ?: return
        
        isRunning.set(true)
        
        // Start packet processing threads
        startPacketProcessing()
        
        // Show notification
        startForeground(NOTIFICATION_ID, createNotification())
        
        // Start statistics update timer
        startStatisticsUpdater()
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
        // Extract destination info from packet (simplified)
        val packetInfo = extractPacketInfo(packet)
        
        if (packetInfo != null && shouldBlockPacket(packetInfo)) {
            // Drop packet by not forwarding it
            return
        }
        
        // Forward allowed packets
        try {
            packet.rewind()
            output.channel.write(packet)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }
    
    private fun extractPacketInfo(packet: ByteBuffer): NetworkPacket? {
        try {
            // Skip ethernet header if present
            if (packet.remaining() < 20) return null
            
            // Parse IP header
            val ipVersion = (packet.get(0).toInt() shr 4) and 0xF
            if (ipVersion != 4) return null // Only support IPv4 for now
            
            // Get protocol (6=TCP, 17=UDP)
            val protocol = packet.get(9).toInt() and 0xFF
            if (protocol != 6 && protocol != 17) return null
            
            // Get destination IP
            val destIp = ByteArray(4)
            packet.position(16)
            packet.get(destIp)
            val destAddress = InetAddress.getByAddress(destIp).hostAddress
            
            // Get IP header length
            val ipHeaderLength = ((packet.get(0).toInt() and 0xF) * 4)
            
            // Parse TCP/UDP header for destination port
            packet.position(ipHeaderLength + 2) // Skip source port
            val destPort = packet.getShort().toInt() and 0xFFFF
            
            // Try to resolve hostname (this is simplified)
            val hostname = destAddress ?: return null
            
            return NetworkPacket(
                host = hostname,
                port = destPort,
                size = packet.remaining()
            )
        } catch (e: Exception) {
            return null
        } finally {
            packet.rewind()
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
        
        val stats = getStatistics()
        val contentText = "ブロック済み: ${stats.blockedCount} | 許可済み: ${stats.allowedCount}"
        
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("AdBlock VPN 保護中")
            .setContentText(contentText)
            .setSmallIcon(android.R.drawable.ic_lock_lock)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }
    
    private fun loadAllFilterLists() {
        // Load saved filter lists
        val savedFilters = filterListManager.loadLocalFilterList()
        if (savedFilters != null) {
            engine.loadFilterList(savedFilters)
        } else {
            // Load default filter rules
            loadDefaultFilterLists()
        }
        
        // Load custom rules
        val customRules = customRulesManager.getAsFilterList()
        if (customRules.isNotEmpty()) {
            engine.loadFilterList(customRules)
        }
    }
    
    private fun loadDefaultFilterLists() {
        // Default filter rules for common ad servers
        val defaultRules = """
            ||doubleclick.net^
            ||googleadservices.com^
            ||googlesyndication.com^
            ||google-analytics.com^
            ||googletagmanager.com^
            ||facebook.com/tr^
            ||amazon-adsystem.com^
            ||adsrvr.org^
            ||adsymptotic.com^
            ||adnxs.com^
            ||adsafeprotected.com^
            ||smaato.net^
            ||smartadserver.com^
            ||scorecardresearch.com^
            ||outbrain.com^
            ||taboola.com^
            ||criteo.com^
            ||criteo.net^
            ||casalemedia.com^
            ||appnexus.com^
            ||rubiconproject.com^
            ||pubmatic.com^
            ||openx.net^
            ||chartboost.com^
            ||unity3d.com/services/ads^
            ||mopub.com^
            ||inmobi.com^
            ||flurry.com^
            ||applovin.com^
            ||startapp.com^
            ||supersonicads.com^
            ||ironsrc.com^
            ||adcolony.com^
            ||vungle.com^
            ||tapjoy.com^
            ||moatads.com^
            ||doubleverify.com^
            ||branch.io^
            ||adjust.com^
            ||kochava.com^
            ||tenjin.io^
            ||singular.net^
            ||appsflyer.com^
            ||crashlytics.com^
            ||fabric.io^
            ||firebase.com/analytics^
            ||mixpanel.com^
            ||segment.com^
            ||amplitude.com^
            ||urbanairship.com^
            ||braze.com^
            ||onesignal.com^
            ||batch.com^
            ||swrve.com^
            ||leanplum.com^
            ||clevertap.com^
            ||airship.com^
            ||mparticle.com^
            ||tune.com^
            ||kochava.com^
            ||youappi.com^
            ||bidmachine.io^
            ||admost.com^
            ||bytedance.com/ad^
            ||tiktok.com/ads^
            
            # YouTube specific rules
            ||youtube.com/api/stats/ads^
            ||youtube.com/pagead^
            ||youtube.com/ptracking^
            ||youtube.com/get_video_info*ad^
            ||youtube.com/api/stats/qoe^
            ||googlevideo.com/videoplayback*ctier^
            ||googlevideo.com/initplayback^
            ||googlevideo.com/ptracking^
            ||googlevideo.com/videogoodput^
            ||youtube.com/youtubei/v1/log_event^
            ||youtube.com/youtubei/v1/player/ad_break^
            ||youtube.com/youtubei/v1/next*adplacements^
            ||youtube.com/youtubei/v1/player*adplacements^
            ||googleads.g.doubleclick.net/pagead/id^
            ||googleads.g.doubleclick.net/pagead/interaction^
            ||static.doubleclick.net/instream/ad_status.js^
            ||2mdn.net/instream^
            ||tpc.googlesyndication.com^
            ||pagead2.googlesyndication.com^
            ||gstatic.com/cast/sdk/libs/ads^
            ||imasdk.googleapis.com^
            ||youtube.com/error_204^
            ||youtube.com/csi_204^
            ||youtube.com/generate_204^
            ||youtube.com/api/stats/watchtime^
            ||youtube.com/api/stats/delayplay^
            ||youtube.com/api/stats/playback^
            ||youtube.com/pcs/activeview^
            ||youtube.com/pagead/paralleladview^
            ||youtube.com/pagead/viewthroughconversion^
            
            # Mobile app ads
            */ads/*
            */adsdk/*
            */advertise/*
            */advertisement/*
            */advertising/*
            */adserver/*
            */adservice/*
            */adnetwork/*
            */analytics/*
            */telemetry/*
            */metrics/*
            */tracking/*
            */banner/*
            */popup/*
            */popunder/*
            */interstitial/*
            */sponsorship/*
            */promoted/*
        """.trimIndent()
        
        engine.loadFilterList(defaultRules)
    }
    
    private fun startStatisticsUpdater() {
        statsUpdateExecutor = Executors.newSingleThreadScheduledExecutor()
        statsUpdateExecutor?.scheduleWithFixedDelay({
            updateStatistics()
        }, 5, 5, TimeUnit.SECONDS)
    }
    
    fun updateStatistics() {
        // Update notification with latest statistics
        if (isRunning.get()) {
            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager.notify(NOTIFICATION_ID, createNotification())
        }
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