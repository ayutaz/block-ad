package com.adblock

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.adblock.ui.theme.AdBlockTheme

class CustomFiltersActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            AdBlockTheme {
                CustomFiltersScreen(
                    onBackPressed = { finish() }
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomFiltersScreen(
    onBackPressed: () -> Unit
) {
    var customRules by remember { mutableStateOf(loadCustomRules()) }
    var showAddDialog by remember { mutableStateOf(false) }
    var newRule by remember { mutableStateOf("") }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("カスタムフィルター") },
                navigationIcon = {
                    IconButton(onClick = onBackPressed) {
                        Icon(Icons.Filled.ArrowBack, contentDescription = "戻る")
                    }
                },
                actions = {
                    IconButton(onClick = { showAddDialog = true }) {
                        Icon(Icons.Filled.Add, contentDescription = "追加")
                    }
                }
            )
        }
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            // 説明カード
            item {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.primaryContainer
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(16.dp)
                    ) {
                        Text(
                            "カスタムルールの書き方",
                            style = MaterialTheme.typography.titleMedium
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            "• ドメインブロック: example.com\n" +
                            "• サブドメインブロック: ||example.com^\n" +
                            "• URLパターン: */ads/*\n" +
                            "• 例外ルール: @@||example.com/allowed",
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                }
            }

            // カスタムルール一覧
            items(customRules) { rule ->
                CustomRuleItem(
                    rule = rule,
                    onDelete = {
                        customRules = customRules.filter { it != rule }
                        saveCustomRules(customRules)
                    }
                )
            }

            // 空の場合のメッセージ
            if (customRules.isEmpty()) {
                item {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(32.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            "カスタムルールはありません",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
        }
    }

    // 追加ダイアログ
    if (showAddDialog) {
        AlertDialog(
            onDismissRequest = { showAddDialog = false },
            title = { Text("新しいルールを追加") },
            text = {
                Column {
                    TextField(
                        value = newRule,
                        onValueChange = { newRule = it },
                        label = { Text("フィルタールール") },
                        placeholder = { Text("例: ||ads.example.com^") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        if (newRule.isNotBlank()) {
                            customRules = customRules + newRule.trim()
                            saveCustomRules(customRules)
                            newRule = ""
                            showAddDialog = false
                        }
                    }
                ) {
                    Text("追加")
                }
            },
            dismissButton = {
                TextButton(onClick = { showAddDialog = false }) {
                    Text("キャンセル")
                }
            }
        )
    }
}

@Composable
fun CustomRuleItem(
    rule: String,
    onDelete: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = rule,
                modifier = Modifier.weight(1f),
                style = MaterialTheme.typography.bodyMedium
            )
            IconButton(onClick = onDelete) {
                Icon(
                    Icons.Filled.Delete,
                    contentDescription = "削除",
                    tint = MaterialTheme.colorScheme.error
                )
            }
        }
    }
}

// Stub functions - will be implemented with actual storage
private fun loadCustomRules(): List<String> {
    // TODO: Load from SharedPreferences
    return emptyList()
}

private fun saveCustomRules(rules: List<String>) {
    // TODO: Save to SharedPreferences
}