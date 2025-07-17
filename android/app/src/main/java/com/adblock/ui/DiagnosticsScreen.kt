package com.adblock.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.adblock.analytics.Analytics
import com.adblock.analytics.CrashReporter
import com.adblock.analytics.CrashType
import com.adblock.analytics.EventCategory
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DiagnosticsScreen(
    onBackPress: () -> Unit
) {
    val context = LocalContext.current
    val crashReporter = remember { CrashReporter.getInstance(context) }
    val analytics = remember { Analytics.getInstance(context) }
    
    var selectedTab by remember { mutableStateOf(0) }
    var showAnalyticsDialog by remember { mutableStateOf(false) }
    var showCrashDialog by remember { mutableStateOf(false) }
    
    val crashReports = remember { crashReporter.getRecentReports(20) }
    val crashStats = remember { crashReporter.getStatistics() }
    val analyticsSummary = remember { analytics.getSummary() }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("診断とレポート") },
                navigationIcon = {
                    IconButton(onClick = onBackPress) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "戻る")
                    }
                },
                actions = {
                    IconButton(onClick = { showAnalyticsDialog = true }) {
                        Icon(Icons.Default.Settings, contentDescription = "設定")
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            // Tab Row
            TabRow(selectedTabIndex = selectedTab) {
                Tab(
                    selected = selectedTab == 0,
                    onClick = { selectedTab = 0 },
                    text = { Text("クラッシュレポート") }
                )
                Tab(
                    selected = selectedTab == 1,
                    onClick = { selectedTab = 1 },
                    text = { Text("使用統計") }
                )
            }
            
            // Content
            when (selectedTab) {
                0 -> CrashReportsTab(
                    crashReports = crashReports,
                    crashStats = crashStats,
                    onClearReports = {
                        crashReporter.clearAllReports()
                    }
                )
                1 -> AnalyticsTab(
                    summary = analyticsSummary,
                    onClearData = {
                        analytics.clearAllData()
                    }
                )
            }
        }
    }
    
    // Analytics Settings Dialog
    if (showAnalyticsDialog) {
        AnalyticsSettingsDialog(
            isEnabled = analytics.isEnabled,
            onEnabledChange = { analytics.isEnabled = it },
            onDismiss = { showAnalyticsDialog = false }
        )
    }
    
    // Crash Reporting Settings Dialog
    if (showCrashDialog) {
        CrashReportingSettingsDialog(
            isEnabled = crashReporter.isEnabled,
            onEnabledChange = { crashReporter.isEnabled = it },
            onDismiss = { showCrashDialog = false }
        )
    }
}

@Composable
fun CrashReportsTab(
    crashReports: List<com.adblock.analytics.CrashReport>,
    crashStats: com.adblock.analytics.CrashStatistics,
    onClearReports: () -> Unit
) {
    LazyColumn(
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        // Statistics Card
        item {
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = if (crashStats.totalCrashes == 0) {
                        Color(0xFF4CAF50).copy(alpha = 0.1f)
                    } else {
                        MaterialTheme.colorScheme.errorContainer
                    }
                )
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        "クラッシュ統計",
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold
                    )
                    
                    Spacer(modifier = Modifier.height(8.dp))
                    
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text("合計クラッシュ数:")
                        Text(
                            crashStats.totalCrashes.toString(),
                            fontWeight = FontWeight.Bold,
                            color = if (crashStats.totalCrashes == 0) {
                                Color(0xFF4CAF50)
                            } else {
                                MaterialTheme.colorScheme.error
                            }
                        )
                    }
                    
                    if (crashStats.crashesByType.isNotEmpty()) {
                        Spacer(modifier = Modifier.height(8.dp))
                        Divider()
                        Spacer(modifier = Modifier.height(8.dp))
                        
                        crashStats.crashesByType.forEach { (type, count) ->
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text(getCrashTypeLabel(type))
                                Text(count.toString())
                            }
                        }
                    }
                    
                    if (crashStats.totalCrashes > 0) {
                        Spacer(modifier = Modifier.height(16.dp))
                        
                        OutlinedButton(
                            onClick = onClearReports,
                            modifier = Modifier.fillMaxWidth(),
                            colors = ButtonDefaults.outlinedButtonColors(
                                contentColor = MaterialTheme.colorScheme.error
                            )
                        ) {
                            Icon(Icons.Default.Delete, contentDescription = null)
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("すべてクリア")
                        }
                    }
                }
            }
        }
        
        // Crash Reports List
        if (crashReports.isNotEmpty()) {
            item {
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    "最近のクラッシュ",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
            }
            
            items(crashReports) { report ->
                CrashReportCard(report)
            }
        } else {
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(32.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Icon(
                            Icons.Default.CheckCircle,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = Color(0xFF4CAF50)
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            "クラッシュレポートはありません",
                            style = MaterialTheme.typography.bodyLarge,
                            color = Color(0xFF4CAF50)
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun CrashReportCard(report: com.adblock.analytics.CrashReport) {
    var expanded by remember { mutableStateOf(false) }
    
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { expanded = !expanded }
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        getCrashTypeLabel(report.type),
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.error
                    )
                    Text(
                        report.message.take(50) + if (report.message.length > 50) "..." else "",
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
                
                Icon(
                    if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                    contentDescription = null
                )
            }
            
            Text(
                SimpleDateFormat("MM/dd HH:mm", Locale.getDefault())
                    .format(Date(report.timestamp)),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            if (expanded) {
                Spacer(modifier = Modifier.height(8.dp))
                Divider()
                Spacer(modifier = Modifier.height(8.dp))
                
                // Details
                DetailRow("バージョン", report.appVersion)
                DetailRow("OS", report.osVersion)
                DetailRow("デバイス", report.deviceModel)
                
                report.context.memoryUsageMB?.let {
                    DetailRow("メモリ使用量", "${it}MB")
                }
                
                report.context.filterRulesCount?.let {
                    DetailRow("フィルタールール数", it.toString())
                }
                
                if (!report.stackTrace.isNullOrEmpty()) {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        "スタックトレース",
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        report.stackTrace.take(300) + "...",
                        style = MaterialTheme.typography.bodySmall,
                        fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                    )
                }
            }
        }
    }
}

@Composable
fun AnalyticsTab(
    summary: com.adblock.analytics.AnalyticsSummary,
    onClearData: () -> Unit
) {
    LazyColumn(
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        // Summary Card
        item {
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    modifier = Modifier.padding(16.dp)
                ) {
                    Text(
                        "使用統計サマリー",
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold
                    )
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceEvenly
                    ) {
                        StatBox(
                            label = "イベント",
                            value = summary.totalEvents.toString()
                        )
                        StatBox(
                            label = "セッション時間",
                            value = "${summary.sessionDuration / 60}分"
                        )
                    }
                }
            }
        }
        
        // Events by Category
        if (summary.eventsByCategory.isNotEmpty()) {
            item {
                Card(
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp)
                    ) {
                        Text(
                            "カテゴリー別イベント",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold
                        )
                        
                        Spacer(modifier = Modifier.height(8.dp))
                        
                        summary.eventsByCategory.forEach { (category, count) ->
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text(getEventCategoryLabel(category))
                                Text(count.toString())
                            }
                            Spacer(modifier = Modifier.height(4.dp))
                        }
                    }
                }
            }
        }
        
        // Metrics
        if (summary.metrics.isNotEmpty()) {
            item {
                Card(
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp)
                    ) {
                        Text(
                            "メトリクス",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold
                        )
                        
                        Spacer(modifier = Modifier.height(8.dp))
                        
                        summary.metrics.forEach { (name, value) ->
                            Column {
                                Text(
                                    getMetricLabel(name),
                                    style = MaterialTheme.typography.labelMedium
                                )
                                Text(
                                    value,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Spacer(modifier = Modifier.height(8.dp))
                            }
                        }
                    }
                }
            }
        }
        
        // Clear Data Button
        item {
            OutlinedButton(
                onClick = onClearData,
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(Icons.Default.Delete, contentDescription = null)
                Spacer(modifier = Modifier.width(8.dp))
                Text("データをクリア")
            }
        }
    }
}

@Composable
fun DetailRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Text(
            value,
            style = MaterialTheme.typography.bodySmall
        )
    }
}

@Composable
fun StatBox(label: String, value: String) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            value,
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold
        )
        Text(
            label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
fun AnalyticsSettingsDialog(
    isEnabled: Boolean,
    onEnabledChange: (Boolean) -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("使用統計設定") },
        text = {
            Column {
                Text("匿名の使用統計を収集してアプリの改善に役立てます。個人情報は一切収集されません。")
                
                Spacer(modifier = Modifier.height(16.dp))
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("使用統計を有効にする")
                    Switch(
                        checked = isEnabled,
                        onCheckedChange = onEnabledChange
                    )
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text("閉じる")
            }
        }
    )
}

@Composable
fun CrashReportingSettingsDialog(
    isEnabled: Boolean,
    onEnabledChange: (Boolean) -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("クラッシュレポート設定") },
        text = {
            Column {
                Text("クラッシュレポートを自動的に収集してアプリの安定性向上に役立てます。個人情報は含まれません。")
                
                Spacer(modifier = Modifier.height(16.dp))
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("クラッシュレポートを有効にする")
                    Switch(
                        checked = isEnabled,
                        onCheckedChange = onEnabledChange
                    )
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text("閉じる")
            }
        }
    )
}

private fun getCrashTypeLabel(type: CrashType): String {
    return when (type) {
        CrashType.NATIVE_CRASH -> "ネイティブクラッシュ"
        CrashType.EXCEPTION -> "例外"
        CrashType.OUT_OF_MEMORY -> "メモリ不足"
        CrashType.ANR -> "応答なし"
        CrashType.NETWORK_ERROR -> "ネットワークエラー"
        CrashType.FILTER_ERROR -> "フィルターエラー"
        CrashType.OTHER -> "その他"
    }
}

private fun getEventCategoryLabel(category: EventCategory): String {
    return when (category) {
        EventCategory.LIFECYCLE -> "ライフサイクル"
        EventCategory.ACTION -> "アクション"
        EventCategory.PERFORMANCE -> "パフォーマンス"
        EventCategory.ERROR -> "エラー"
        EventCategory.FEATURE -> "機能"
    }
}

private fun getMetricLabel(name: String): String {
    return when (name) {
        "ads_blocked" -> "ブロックした広告"
        "bytes_saved" -> "節約したデータ"
        "vpn_connections" -> "VPN接続回数"
        "custom_rules" -> "カスタムルール"
        else -> name
    }
}