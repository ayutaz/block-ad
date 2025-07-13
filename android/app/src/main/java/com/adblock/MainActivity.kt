package com.adblock

import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.lifecycle.lifecycleScope
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.outlined.Security
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.adblock.ui.theme.AdBlockTheme
import com.adblock.vpn.AdBlockVpnService
import com.adblock.filter.FilterListManager
import com.adblock.filter.CustomRulesManager
import com.adblock.worker.FilterUpdateWorker
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Locale

class MainActivity : ComponentActivity() {
    
    private lateinit var filterListManager: FilterListManager
    private lateinit var customRulesManager: CustomRulesManager
    private lateinit var adBlockEngine: AdBlockEngine
    
    private val vpnPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == RESULT_OK) {
            startVpnService()
        }
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize components
        filterListManager = FilterListManager(this)
        customRulesManager = CustomRulesManager(this)
        adBlockEngine = AdBlockEngine.getInstance()
        
        // Load local filter list if available
        loadLocalFilters()
        
        // Schedule automatic filter updates if enabled
        if (filterListManager.isAutoUpdateEnabled()) {
            FilterUpdateWorker.schedulePeriodicUpdate(this)
        }
        
        setContent {
            AdBlockTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    MainScreen(
                        onToggleVpn = { enabled ->
                            if (enabled) {
                                checkVpnPermission()
                            } else {
                                stopVpnService()
                            }
                        },
                        onUpdateFilterLists = {
                            updateFilterLists()
                        },
                        onClearStatistics = {
                            clearStatistics()
                        },
                        onOpenCustomFilters = {
                            val intent = Intent(this, CustomFiltersActivity::class.java)
                            startActivity(intent)
                        }
                    )
                }
            }
        }
    }
    
    private fun checkVpnPermission() {
        val intent = VpnService.prepare(this)
        if (intent != null) {
            vpnPermissionLauncher.launch(intent)
        } else {
            startVpnService()
        }
    }
    
    private fun startVpnService() {
        val intent = Intent(this, AdBlockVpnService::class.java)
        startService(intent)
    }
    
    private fun stopVpnService() {
        val intent = Intent(this, AdBlockVpnService::class.java)
        stopService(intent)
    }
    
    private fun updateFilterLists() {
        lifecycleScope.launch {
            try {
                // Show loading state
                showToast("フィルターリストを更新中...")
                
                // Update filter lists
                val result = filterListManager.updateFilterLists()
                
                result.fold(
                    onSuccess = { filterContent ->
                        // Load the new filters into the engine
                        val loaded = adBlockEngine.loadFilterList(filterContent)
                        if (loaded) {
                            showToast("フィルターリストを更新しました")
                        } else {
                            showToast("フィルターリストの読み込みに失敗しました")
                        }
                    },
                    onFailure = { error ->
                        showToast("更新に失敗しました: ${error.message}")
                    }
                )
            } catch (e: Exception) {
                showToast("エラーが発生しました: ${e.message}")
            }
        }
    }
    
    private fun clearStatistics() {
        lifecycleScope.launch {
            try {
                // Reset statistics in the engine
                val success = adBlockEngine.resetStatistics()
                if (success) {
                    showToast("統計情報をリセットしました")
                } else {
                    showToast("リセットに失敗しました")
                }
            } catch (e: Exception) {
                showToast("エラーが発生しました: ${e.message}")
            }
        }
    }
    
    private fun loadLocalFilters() {
        lifecycleScope.launch {
            // Load EasyList filters
            val localFilters = filterListManager.loadLocalFilterList()
            if (localFilters != null) {
                adBlockEngine.loadFilterList(localFilters)
            }
            
            // Load custom rules
            val customRules = customRulesManager.getAsFilterList()
            if (customRules.isNotEmpty()) {
                adBlockEngine.loadFilterList(customRules)
            }
        }
    }
    
    private fun showToast(message: String) {
        runOnUiThread {
            android.widget.Toast.makeText(this, message, android.widget.Toast.LENGTH_SHORT).show()
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        adBlockEngine.destroy()
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    onToggleVpn: (Boolean) -> Unit,
    onUpdateFilterLists: () -> Unit,
    onClearStatistics: () -> Unit,
    onOpenCustomFilters: () -> Unit = {}
) {
    var isVpnEnabled by remember { mutableStateOf(false) }
    var statistics by remember { mutableStateOf(Statistics(0, 0, 0)) }
    var showSettings by remember { mutableStateOf(false) }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("AdBlock") },
                actions = {
                    IconButton(onClick = { showSettings = true }) {
                        Icon(
                            imageVector = Icons.Default.Settings,
                            contentDescription = "Settings"
                        )
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Main Toggle Card
            Card(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(24.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Icon(
                        imageVector = if (isVpnEnabled) Icons.Filled.Security else Icons.Outlined.Security,
                        contentDescription = null,
                        modifier = Modifier.size(64.dp),
                        tint = if (isVpnEnabled) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    Text(
                        text = if (isVpnEnabled) "保護中" else "保護されていません",
                        style = MaterialTheme.typography.headlineMedium,
                        color = if (isVpnEnabled) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    Switch(
                        checked = isVpnEnabled,
                        onCheckedChange = { enabled ->
                            isVpnEnabled = enabled
                            onToggleVpn(enabled)
                        },
                        modifier = Modifier.scale(1.5f)
                    )
                }
            }
            
            Spacer(modifier = Modifier.height(24.dp))
            
            // Statistics Card
            Card(
                modifier = Modifier.fillMaxWidth(),
                elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp)
                ) {
                    Text(
                        text = "統計情報",
                        style = MaterialTheme.typography.titleMedium,
                        modifier = Modifier.padding(bottom = 12.dp)
                    )
                    
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceEvenly
                    ) {
                        StatisticItem(
                            label = "ブロック済み",
                            value = statistics.blockedCount.toString(),
                            color = MaterialTheme.colorScheme.error
                        )
                        StatisticItem(
                            label = "許可済み",
                            value = statistics.allowedCount.toString(),
                            color = MaterialTheme.colorScheme.primary
                        )
                        StatisticItem(
                            label = "ブロック率",
                            value = "${statistics.blockRate.toInt()}%",
                            color = MaterialTheme.colorScheme.secondary
                        )
                    }
                    
                    Spacer(modifier = Modifier.height(12.dp))
                    
                    LinearProgressIndicator(
                        progress = statistics.blockRate.toFloat() / 100f,
                        modifier = Modifier.fillMaxWidth()
                    )
                    
                    Text(
                        text = "節約データ量: ${formatDataSize(statistics.dataSaved)}",
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(top = 8.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Quick Actions
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceEvenly
            ) {
                OutlinedButton(
                    onClick = onUpdateFilterLists,
                    modifier = Modifier.weight(1f).padding(end = 8.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Refresh,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("フィルター更新")
                }
                
                OutlinedButton(
                    onClick = onClearStatistics,
                    modifier = Modifier.weight(1f).padding(start = 8.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Clear,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("統計リセット")
                }
            }
            
            Spacer(modifier = Modifier.height(8.dp))
            
            // Custom Filters Button
            OutlinedButton(
                onClick = onOpenCustomFilters,
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(
                    imageVector = Icons.Default.Add,
                    contentDescription = null,
                    modifier = Modifier.size(16.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("カスタムフィルター")
            }
        }
    }
    
    // Settings Dialog
    if (showSettings) {
        SettingsDialog(
            onDismiss = { showSettings = false }
        )
    }
}

@Composable
fun StatisticItem(
    label: String,
    value: String,
    color: Color
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = value,
            style = MaterialTheme.typography.headlineSmall,
            color = color
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
fun SettingsDialog(
    onDismiss: () -> Unit
) {
    val context = LocalContext.current
    val filterListManager = remember { FilterListManager(context) }
    var autoUpdate by remember { mutableStateOf(filterListManager.isAutoUpdateEnabled()) }
    var blockYouTube by remember { mutableStateOf(true) }
    
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("設定") },
        text = {
            Column {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("フィルターの自動更新")
                    Switch(
                        checked = autoUpdate,
                        onCheckedChange = { 
                            autoUpdate = it
                            filterListManager.setAutoUpdateEnabled(it)
                            if (it) {
                                FilterUpdateWorker.schedulePeriodicUpdate(context)
                            } else {
                                FilterUpdateWorker.cancelAllUpdates(context)
                            }
                        }
                    )
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("YouTube広告ブロック")
                    Switch(
                        checked = blockYouTube,
                        onCheckedChange = { blockYouTube = it }
                    )
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                Text(
                    text = "バージョン: 1.0.0",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text("閉じる")
            }
        }
    )
}

fun formatDataSize(bytes: Int): String {
    return when {
        bytes < 1024 -> "$bytes B"
        bytes < 1024 * 1024 -> "${bytes / 1024} KB"
        bytes < 1024 * 1024 * 1024 -> "${bytes / (1024 * 1024)} MB"
        else -> "${bytes / (1024 * 1024 * 1024)} GB"
    }
}