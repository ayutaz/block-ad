package com.adblock.ui

import androidx.compose.animation.core.*
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.adblock.AdBlockEngine
import com.adblock.PerformanceMetrics
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import java.text.DecimalFormat
import kotlin.math.min

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PerformanceScreen(
    onBack: () -> Unit
) {
    val engine = AdBlockEngine.getInstance()
    var metrics by remember { mutableStateOf<PerformanceMetrics?>(null) }
    var isRefreshing by remember { mutableStateOf(false) }
    
    // Auto-refresh metrics every 2 seconds
    LaunchedEffect(Unit) {
        while (isActive) {
            metrics = engine.getPerformanceMetrics()
            delay(2000)
        }
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("パフォーマンス監視") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "戻る")
                    }
                },
                actions = {
                    IconButton(
                        onClick = {
                            isRefreshing = true
                            metrics = engine.getPerformanceMetrics()
                            isRefreshing = false
                        }
                    ) {
                        Icon(
                            Icons.Default.Refresh,
                            contentDescription = "更新",
                            tint = if (isRefreshing) MaterialTheme.colorScheme.primary
                            else MaterialTheme.colorScheme.onSurface
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
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            metrics?.let { perf ->
                // Overview Cards
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    MetricCard(
                        modifier = Modifier.weight(1f),
                        title = "平均処理時間",
                        value = formatTime(perf.avgProcessingTime),
                        icon = Icons.Default.Speed,
                        color = MaterialTheme.colorScheme.primary
                    )
                    
                    MetricCard(
                        modifier = Modifier.weight(1f),
                        title = "メモリ使用量",
                        value = formatMemory(perf.memoryUsage),
                        icon = Icons.Default.Memory,
                        color = MaterialTheme.colorScheme.secondary
                    )
                }
                
                // Request Statistics
                RequestStatsCard(metrics = perf)
                
                // Performance Chart
                PerformanceChartCard(metrics = perf)
                
                // CPU Usage
                CpuUsageCard(cpuUsage = perf.cpuUsage)
                
                // Cache Statistics
                CacheStatsCard(metrics = perf)
                
                // Detailed Metrics
                DetailedMetricsCard(metrics = perf)
            } ?: run {
                // Loading state
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(32.dp),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            }
        }
    }
}

@Composable
fun MetricCard(
    modifier: Modifier = Modifier,
    title: String,
    value: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    color: Color
) {
    Card(
        modifier = modifier,
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(32.dp),
                tint = color
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = value,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                color = color
            )
            Text(
                text = title,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun RequestStatsCard(metrics: PerformanceMetrics) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                "リクエスト統計",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
            
            Spacer(modifier = Modifier.height(16.dp))
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceEvenly
            ) {
                StatItem(
                    label = "総リクエスト",
                    value = metrics.totalRequests.toString(),
                    color = MaterialTheme.colorScheme.primary
                )
                StatItem(
                    label = "ブロック済み",
                    value = metrics.blockedRequests.toString(),
                    color = MaterialTheme.colorScheme.error
                )
                StatItem(
                    label = "許可済み",
                    value = metrics.allowedRequests.toString(),
                    color = MaterialTheme.colorScheme.tertiary
                )
            }
            
            Spacer(modifier = Modifier.height(12.dp))
            
            // Block rate progress bar
            val blockRate = if (metrics.totalRequests > 0) {
                (metrics.blockedRequests.toFloat() / metrics.totalRequests) * 100
            } else 0f
            
            Column {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        "ブロック率",
                        style = MaterialTheme.typography.bodyMedium
                    )
                    Text(
                        "${blockRate.toInt()}%",
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = FontWeight.Bold
                    )
                }
                Spacer(modifier = Modifier.height(4.dp))
                LinearProgressIndicator(
                    progress = blockRate / 100f,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(8.dp)
                        .clip(RoundedCornerShape(4.dp)),
                    color = MaterialTheme.colorScheme.error,
                    trackColor = MaterialTheme.colorScheme.surfaceVariant
                )
            }
        }
    }
}

@Composable
fun StatItem(
    label: String,
    value: String,
    color: Color
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = value,
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.Bold,
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
fun PerformanceChartCard(metrics: PerformanceMetrics) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                "処理時間分布",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Processing time distribution
            Canvas(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(200.dp)
            ) {
                drawProcessingTimeChart(
                    p50 = metrics.p50ProcessingTime,
                    p95 = metrics.p95ProcessingTime,
                    p99 = metrics.p99ProcessingTime,
                    max = metrics.maxProcessingTime
                )
            }
            
            Spacer(modifier = Modifier.height(12.dp))
            
            // Legend
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceEvenly
            ) {
                LegendItem("P50", formatTime(metrics.p50ProcessingTime), Color(0xFF4CAF50))
                LegendItem("P95", formatTime(metrics.p95ProcessingTime), Color(0xFFFFC107))
                LegendItem("P99", formatTime(metrics.p99ProcessingTime), Color(0xFFFF9800))
                LegendItem("Max", formatTime(metrics.maxProcessingTime), Color(0xFFF44336))
            }
        }
    }
}

@Composable
fun LegendItem(label: String, value: String, color: Color) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier
                .size(12.dp)
                .background(color, RoundedCornerShape(2.dp))
        )
        Spacer(modifier = Modifier.width(4.dp))
        Column {
            Text(
                label,
                style = MaterialTheme.typography.bodySmall,
                fontWeight = FontWeight.Bold
            )
            Text(
                value,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun CpuUsageCard(cpuUsage: Float) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                "CPU使用率",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Circular progress for CPU usage
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier.size(120.dp)
            ) {
                CircularProgressIndicator(
                    progress = cpuUsage / 100f,
                    modifier = Modifier.fillMaxSize(),
                    strokeWidth = 12.dp,
                    color = when {
                        cpuUsage > 80 -> Color.Red
                        cpuUsage > 50 -> Color(0xFFFFC107)
                        else -> Color(0xFF4CAF50)
                    },
                    trackColor = MaterialTheme.colorScheme.surfaceVariant
                )
                
                Text(
                    text = "${cpuUsage.toInt()}%",
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold
                )
            }
            
            Spacer(modifier = Modifier.height(8.dp))
            
            Text(
                text = when {
                    cpuUsage > 80 -> "高負荷"
                    cpuUsage > 50 -> "中負荷"
                    else -> "低負荷"
                },
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun CacheStatsCard(metrics: PerformanceMetrics) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    "キャッシュ統計",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                
                Icon(
                    Icons.Default.Storage,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary
                )
            }
            
            Spacer(modifier = Modifier.height(12.dp))
            
            val hitRate = if (metrics.cacheHits + metrics.cacheMisses > 0) {
                (metrics.cacheHits.toFloat() / (metrics.cacheHits + metrics.cacheMisses)) * 100
            } else 0f
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceEvenly
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        metrics.cacheHits.toString(),
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                        color = Color(0xFF4CAF50)
                    )
                    Text(
                        "ヒット",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        metrics.cacheMisses.toString(),
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                        color = Color(0xFFF44336)
                    )
                    Text(
                        "ミス",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        "${hitRate.toInt()}%",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.primary
                    )
                    Text(
                        "ヒット率",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

@Composable
fun DetailedMetricsCard(metrics: PerformanceMetrics) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp)
        ) {
            Text(
                "詳細メトリクス",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
            
            Spacer(modifier = Modifier.height(12.dp))
            
            MetricRow("フィルタールール数", metrics.filterRuleCount.toString())
            MetricRow("最終更新", formatTimestamp(metrics.lastUpdateTime))
            MetricRow("稼働時間", formatUptime(metrics.uptimeSeconds))
            MetricRow("エラー数", metrics.errorCount.toString())
        }
    }
}

@Composable
fun MetricRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            label,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Text(
            value,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.Medium
        )
    }
}

// Helper functions
private fun formatTime(nanos: Long): String {
    return when {
        nanos < 1_000 -> "${nanos}ns"
        nanos < 1_000_000 -> "${nanos / 1_000}μs"
        nanos < 1_000_000_000 -> "${nanos / 1_000_000}ms"
        else -> "${nanos / 1_000_000_000}s"
    }
}

private fun formatMemory(bytes: Long): String {
    val df = DecimalFormat("#.#")
    return when {
        bytes < 1024 -> "$bytes B"
        bytes < 1024 * 1024 -> "${df.format(bytes / 1024.0)} KB"
        bytes < 1024 * 1024 * 1024 -> "${df.format(bytes / (1024.0 * 1024))} MB"
        else -> "${df.format(bytes / (1024.0 * 1024 * 1024))} GB"
    }
}

private fun formatTimestamp(timestamp: Long): String {
    val now = System.currentTimeMillis()
    val diff = now - timestamp
    
    return when {
        diff < 60_000 -> "たった今"
        diff < 3600_000 -> "${diff / 60_000}分前"
        diff < 86400_000 -> "${diff / 3600_000}時間前"
        else -> "${diff / 86400_000}日前"
    }
}

private fun formatUptime(seconds: Long): String {
    val days = seconds / 86400
    val hours = (seconds % 86400) / 3600
    val minutes = (seconds % 3600) / 60
    
    return when {
        days > 0 -> "${days}日 ${hours}時間"
        hours > 0 -> "${hours}時間 ${minutes}分"
        else -> "${minutes}分"
    }
}

private fun DrawScope.drawProcessingTimeChart(
    p50: Long,
    p95: Long,
    p99: Long,
    max: Long
) {
    val maxValue = max.toFloat()
    val barWidth = size.width / 8
    val spacing = size.width / 5
    
    val values = listOf(
        p50 to Color(0xFF4CAF50),
        p95 to Color(0xFFFFC107),
        p99 to Color(0xFFFF9800),
        max to Color(0xFFF44336)
    )
    
    values.forEachIndexed { index, (value, color) ->
        val height = (value.toFloat() / maxValue) * size.height * 0.8f
        val x = spacing * (index + 1) - barWidth / 2
        
        drawRect(
            color = color,
            topLeft = Offset(x, size.height - height),
            size = Size(barWidth, height)
        )
    }
}