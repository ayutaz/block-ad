package com.adblock.testing

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.adblock.ui.theme.AdBlockTheme
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.*

/**
 * Activity for testing YouTube ad blocking effectiveness
 * This is a debug/testing tool not included in release builds
 */
class YouTubeAdTestActivity : ComponentActivity() {
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        setContent {
            AdBlockTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    YouTubeAdTestScreen(
                        onBack = { finish() }
                    )
                }
            }
        }
    }
}

data class VideoTestResult(
    val id: String = UUID.randomUUID().toString(),
    val videoTitle: String,
    val channelName: String,
    val category: String,
    val preRollAdShown: Boolean,
    val midRollAdsShown: Int,
    val midRollAdsTotal: Int,
    val postRollAdShown: Boolean,
    val testTime: Long = System.currentTimeMillis(),
    val notes: String = ""
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun YouTubeAdTestScreen(
    onBack: () -> Unit
) {
    var testResults by remember { mutableStateOf(listOf<VideoTestResult>()) }
    var showAddDialog by remember { mutableStateOf(false) }
    var selectedCategory by remember { mutableStateOf("All") }
    
    val categories = listOf(
        "All", "Music", "Gaming", "Tech", "News", "Education", "Entertainment", "Sports"
    )
    
    val filteredResults = if (selectedCategory == "All") {
        testResults
    } else {
        testResults.filter { it.category == selectedCategory }
    }
    
    // Calculate statistics
    val totalVideos = filteredResults.size
    val totalPreRollAds = filteredResults.count { it.preRollAdShown }
    val totalMidRollAds = filteredResults.sumOf { it.midRollAdsShown }
    val totalPossibleMidRollAds = filteredResults.sumOf { it.midRollAdsTotal }
    val totalPostRollAds = filteredResults.count { it.postRollAdShown }
    
    val totalAdsShown = totalPreRollAds + totalMidRollAds + totalPostRollAds
    val totalPossibleAds = totalVideos + totalPossibleMidRollAds + totalVideos
    
    val blockRate = if (totalPossibleAds > 0) {
        ((totalPossibleAds - totalAdsShown).toFloat() / totalPossibleAds * 100)
    } else 0f
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("YouTube広告ブロックテスト") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "戻る")
                    }
                },
                actions = {
                    IconButton(onClick = { showAddDialog = true }) {
                        Icon(Icons.Default.Add, contentDescription = "テスト追加")
                    }
                    
                    IconButton(onClick = { 
                        // Export results
                        exportResults(testResults)
                    }) {
                        Icon(Icons.Default.Share, contentDescription = "エクスポート")
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
            // Statistics Card
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                colors = CardDefaults.cardColors(
                    containerColor = if (blockRate >= 80) {
                        Color(0xFF4CAF50).copy(alpha = 0.1f)
                    } else {
                        MaterialTheme.colorScheme.errorContainer
                    }
                )
            ) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Text(
                        "ブロック率",
                        style = MaterialTheme.typography.titleMedium
                    )
                    
                    Text(
                        "${blockRate.toInt()}%",
                        style = MaterialTheme.typography.displayLarge,
                        fontWeight = FontWeight.Bold,
                        color = if (blockRate >= 80) Color(0xFF4CAF50) else MaterialTheme.colorScheme.error
                    )
                    
                    Spacer(modifier = Modifier.height(8.dp))
                    
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceEvenly
                    ) {
                        StatItem("テスト動画", totalVideos.toString())
                        StatItem("表示広告", totalAdsShown.toString())
                        StatItem("ブロック", (totalPossibleAds - totalAdsShown).toString())
                    }
                    
                    if (blockRate < 80) {
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            "⚠️ 目標の80%に達していません",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.error
                        )
                    }
                }
            }
            
            // Category Filter
            ScrollableTabRow(
                selectedTabIndex = categories.indexOf(selectedCategory),
                modifier = Modifier.fillMaxWidth()
            ) {
                categories.forEach { category ->
                    Tab(
                        selected = selectedCategory == category,
                        onClick = { selectedCategory = category },
                        text = { Text(category) }
                    )
                }
            }
            
            // Results List
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(filteredResults) { result ->
                    TestResultCard(result)
                }
                
                if (filteredResults.isEmpty()) {
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
                                    Icons.Default.VideoLibrary,
                                    contentDescription = null,
                                    modifier = Modifier.size(64.dp),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Spacer(modifier = Modifier.height(16.dp))
                                Text(
                                    "テスト結果がありません",
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Spacer(modifier = Modifier.height(8.dp))
                                Text(
                                    "＋ボタンから新しいテストを追加",
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    }
                }
            }
        }
    }
    
    if (showAddDialog) {
        AddTestResultDialog(
            onDismiss = { showAddDialog = false },
            onAdd = { result ->
                testResults = testResults + result
                showAddDialog = false
            }
        )
    }
}

@Composable
fun StatItem(
    label: String,
    value: String
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            value,
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.Bold
        )
        Text(
            label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@Composable
fun TestResultCard(result: VideoTestResult) {
    Card(
        modifier = Modifier.fillMaxWidth()
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
                        result.videoTitle,
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        result.channelName,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                AssistChip(
                    onClick = { },
                    label = { Text(result.category) }
                )
            }
            
            Spacer(modifier = Modifier.height(8.dp))
            
            // Ad indicators
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                AdChip(
                    label = "Pre-roll",
                    shown = result.preRollAdShown
                )
                
                if (result.midRollAdsTotal > 0) {
                    AdChip(
                        label = "Mid-roll ${result.midRollAdsShown}/${result.midRollAdsTotal}",
                        shown = result.midRollAdsShown > 0
                    )
                }
                
                AdChip(
                    label = "Post-roll",
                    shown = result.postRollAdShown
                )
            }
            
            if (result.notes.isNotEmpty()) {
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    result.notes,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            
            Spacer(modifier = Modifier.height(4.dp))
            
            Text(
                SimpleDateFormat("yyyy/MM/dd HH:mm", Locale.getDefault())
                    .format(Date(result.testTime)),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun AdChip(
    label: String,
    shown: Boolean
) {
    val color = if (shown) MaterialTheme.colorScheme.error else Color(0xFF4CAF50)
    val icon = if (shown) Icons.Default.Cancel else Icons.Default.CheckCircle
    
    AssistChip(
        onClick = { },
        label = { Text(label) },
        leadingIcon = {
            Icon(
                icon,
                contentDescription = null,
                modifier = Modifier.size(16.dp),
                tint = color
            )
        },
        colors = AssistChipDefaults.assistChipColors(
            containerColor = color.copy(alpha = 0.1f),
            labelColor = color
        )
    )
}

@Composable
fun AddTestResultDialog(
    onDismiss: () -> Unit,
    onAdd: (VideoTestResult) -> Unit
) {
    var videoTitle by remember { mutableStateOf("") }
    var channelName by remember { mutableStateOf("") }
    var category by remember { mutableStateOf("Music") }
    var preRollAdShown by remember { mutableStateOf(false) }
    var midRollAdsShown by remember { mutableStateOf("0") }
    var midRollAdsTotal by remember { mutableStateOf("0") }
    var postRollAdShown by remember { mutableStateOf(false) }
    var notes by remember { mutableStateOf("") }
    
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("テスト結果を追加") },
        text = {
            Column(
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                OutlinedTextField(
                    value = videoTitle,
                    onValueChange = { videoTitle = it },
                    label = { Text("動画タイトル") },
                    modifier = Modifier.fillMaxWidth()
                )
                
                OutlinedTextField(
                    value = channelName,
                    onValueChange = { channelName = it },
                    label = { Text("チャンネル名") },
                    modifier = Modifier.fillMaxWidth()
                )
                
                // Category dropdown
                var expanded by remember { mutableStateOf(false) }
                ExposedDropdownMenuBox(
                    expanded = expanded,
                    onExpandedChange = { expanded = !expanded }
                ) {
                    OutlinedTextField(
                        value = category,
                        onValueChange = { },
                        readOnly = true,
                        label = { Text("カテゴリー") },
                        trailingIcon = {
                            ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded)
                        },
                        modifier = Modifier
                            .fillMaxWidth()
                            .menuAnchor()
                    )
                    
                    ExposedDropdownMenu(
                        expanded = expanded,
                        onDismissRequest = { expanded = false }
                    ) {
                        listOf("Music", "Gaming", "Tech", "News", "Education", "Entertainment", "Sports")
                            .forEach { cat ->
                                DropdownMenuItem(
                                    text = { Text(cat) },
                                    onClick = {
                                        category = cat
                                        expanded = false
                                    }
                                )
                            }
                    }
                }
                
                // Ad checkboxes
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Checkbox(
                        checked = preRollAdShown,
                        onCheckedChange = { preRollAdShown = it }
                    )
                    Text("Pre-roll広告が表示された")
                }
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    OutlinedTextField(
                        value = midRollAdsShown,
                        onValueChange = { midRollAdsShown = it },
                        label = { Text("Mid-roll表示") },
                        modifier = Modifier.weight(1f)
                    )
                    
                    OutlinedTextField(
                        value = midRollAdsTotal,
                        onValueChange = { midRollAdsTotal = it },
                        label = { Text("Mid-roll総数") },
                        modifier = Modifier.weight(1f)
                    )
                }
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Checkbox(
                        checked = postRollAdShown,
                        onCheckedChange = { postRollAdShown = it }
                    )
                    Text("Post-roll広告が表示された")
                }
                
                OutlinedTextField(
                    value = notes,
                    onValueChange = { notes = it },
                    label = { Text("メモ（任意）") },
                    modifier = Modifier.fillMaxWidth()
                )
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    val result = VideoTestResult(
                        videoTitle = videoTitle,
                        channelName = channelName,
                        category = category,
                        preRollAdShown = preRollAdShown,
                        midRollAdsShown = midRollAdsShown.toIntOrNull() ?: 0,
                        midRollAdsTotal = midRollAdsTotal.toIntOrNull() ?: 0,
                        postRollAdShown = postRollAdShown,
                        notes = notes
                    )
                    onAdd(result)
                },
                enabled = videoTitle.isNotBlank() && channelName.isNotBlank()
            ) {
                Text("追加")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("キャンセル")
            }
        }
    )
}

private fun exportResults(results: List<VideoTestResult>) {
    // Create CSV export
    val csv = buildString {
        appendLine("Video Title,Channel,Category,Pre-roll Ad,Mid-roll Shown,Mid-roll Total,Post-roll Ad,Test Time,Notes")
        
        results.forEach { result ->
            appendLine(
                "${result.videoTitle}," +
                "${result.channelName}," +
                "${result.category}," +
                "${result.preRollAdShown}," +
                "${result.midRollAdsShown}," +
                "${result.midRollAdsTotal}," +
                "${result.postRollAdShown}," +
                "${SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.getDefault()).format(Date(result.testTime))}," +
                "\"${result.notes}\""
            )
        }
        
        appendLine()
        appendLine("Summary")
        appendLine("Total Videos,${results.size}")
        
        val totalAdsShown = results.count { it.preRollAdShown } + 
                           results.sumOf { it.midRollAdsShown } + 
                           results.count { it.postRollAdShown }
        val totalPossibleAds = results.size + 
                              results.sumOf { it.midRollAdsTotal } + 
                              results.size
        val blockRate = if (totalPossibleAds > 0) {
            ((totalPossibleAds - totalAdsShown).toFloat() / totalPossibleAds * 100)
        } else 0f
        
        appendLine("Total Ads Shown,$totalAdsShown")
        appendLine("Total Ads Blocked,${totalPossibleAds - totalAdsShown}")
        appendLine("Block Rate,${blockRate}%")
    }
    
    // In a real implementation, share this CSV via Intent
    println(csv)
}