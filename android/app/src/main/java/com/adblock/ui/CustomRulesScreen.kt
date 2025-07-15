package com.adblock.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.adblock.AdBlockEngine
import com.adblock.filter.CustomRulesManager

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomRulesScreen(
    onBack: () -> Unit
) {
    val context = LocalContext.current
    val customRulesManager = remember { CustomRulesManager(context) }
    var rules by remember { mutableStateOf(customRulesManager.loadRules()) }
    var showAddDialog by remember { mutableStateOf(false) }
    var showImportDialog by remember { mutableStateOf(false) }
    var selectedRule by remember { mutableStateOf<String?>(null) }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("カスタムフィルタールール") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "戻る")
                    }
                },
                actions = {
                    IconButton(onClick = { showImportDialog = true }) {
                        Icon(Icons.Default.FileDownload, contentDescription = "インポート")
                    }
                    IconButton(onClick = { showAddDialog = true }) {
                        Icon(Icons.Default.Add, contentDescription = "追加")
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
            // Info card
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer
                )
            ) {
                Row(
                    modifier = Modifier.padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        Icons.Default.Info,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onPrimaryContainer
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = "EasyList形式のフィルタールールを追加できます",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onPrimaryContainer
                    )
                }
            }
            
            // Rules list
            if (rules.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(32.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Icon(
                            Icons.Default.FilterAlt,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            "カスタムルールがありません",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            "右上の＋ボタンから追加してください",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(rules) { rule ->
                        RuleItem(
                            rule = rule,
                            onClick = { selectedRule = rule }
                        )
                    }
                }
            }
        }
    }
    
    // Add rule dialog
    if (showAddDialog) {
        AddRuleDialog(
            onDismiss = { showAddDialog = false },
            onAdd = { rule ->
                customRulesManager.addRule(rule)
                rules = customRulesManager.loadRules()
                
                // Apply rules to engine
                val engine = AdBlockEngine.getInstance()
                engine.loadFilters(customRulesManager.getAsFilterList())
                
                showAddDialog = false
            }
        )
    }
    
    // Import rules dialog
    if (showImportDialog) {
        ImportRulesDialog(
            onDismiss = { showImportDialog = false },
            onImport = { importedRules ->
                importedRules.forEach { rule ->
                    customRulesManager.addRule(rule)
                }
                rules = customRulesManager.loadRules()
                
                // Apply rules to engine
                val engine = AdBlockEngine.getInstance()
                engine.loadFilters(customRulesManager.getAsFilterList())
                
                showImportDialog = false
            }
        )
    }
    
    // Rule detail dialog
    selectedRule?.let { rule ->
        RuleDetailDialog(
            rule = rule,
            onDismiss = { selectedRule = null },
            onDelete = {
                customRulesManager.removeRule(rule)
                rules = customRulesManager.loadRules()
                
                // Apply rules to engine
                val engine = AdBlockEngine.getInstance()
                engine.loadFilters(customRulesManager.getAsFilterList())
                
                selectedRule = null
            }
        )
    }
}

@Composable
fun RuleItem(
    rule: String,
    onClick: () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() },
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Rule type icon
            val icon = when {
                rule.startsWith("||") && rule.endsWith("^") -> Icons.Default.Block
                rule.startsWith("@@") -> Icons.Default.CheckCircle
                rule.startsWith("##") -> Icons.Default.Code
                else -> Icons.Default.FilterList
            }
            
            Icon(
                icon,
                contentDescription = null,
                tint = when {
                    rule.startsWith("@@") -> Color(0xFF4CAF50)
                    else -> MaterialTheme.colorScheme.primary
                }
            )
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = rule,
                    style = MaterialTheme.typography.bodyMedium,
                    fontFamily = FontFamily.Monospace,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                
                // Rule type description
                val description = when {
                    rule.startsWith("||") && rule.endsWith("^") -> "ドメインブロック"
                    rule.startsWith("@@") -> "例外ルール"
                    rule.startsWith("##") -> "要素非表示"
                    rule.contains("*") -> "ワイルドカードルール"
                    else -> "カスタムルール"
                }
                
                Text(
                    text = description,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            
            Icon(
                Icons.Default.ChevronRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun AddRuleDialog(
    onDismiss: () -> Unit,
    onAdd: (String) -> Unit
) {
    var ruleText by remember { mutableStateOf("") }
    var ruleType by remember { mutableStateOf("domain") }
    
    Dialog(onDismissRequest = onDismiss) {
        Card(
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(16.dp)
        ) {
            Column(
                modifier = Modifier.padding(24.dp)
            ) {
                Text(
                    "フィルタールールを追加",
                    style = MaterialTheme.typography.headlineSmall
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                // Rule type selector
                Column {
                    Text(
                        "ルールタイプ:",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    Spacer(modifier = Modifier.height(8.dp))
                    
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        FilterChip(
                            selected = ruleType == "domain",
                            onClick = { ruleType = "domain" },
                            label = { Text("ドメイン") }
                        )
                        FilterChip(
                            selected = ruleType == "exception",
                            onClick = { ruleType = "exception" },
                            label = { Text("例外") }
                        )
                        FilterChip(
                            selected = ruleType == "custom",
                            onClick = { ruleType = "custom" },
                            label = { Text("カスタム") }
                        )
                    }
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                // Rule input
                OutlinedTextField(
                    value = ruleText,
                    onValueChange = { ruleText = it },
                    label = {
                        Text(
                            when (ruleType) {
                                "domain" -> "ドメイン (例: example.com)"
                                "exception" -> "例外URL (例: example.com/allowed)"
                                else -> "フィルタールール"
                            }
                        )
                    },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = false,
                    minLines = 2
                )
                
                // Rule format hint
                if (ruleType != "custom") {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = when (ruleType) {
                            "domain" -> "入力されたドメインは ||domain^ 形式に変換されます"
                            "exception" -> "入力されたURLは @@||url 形式に変換されます"
                            else -> ""
                        },
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                Spacer(modifier = Modifier.height(24.dp))
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End
                ) {
                    TextButton(onClick = onDismiss) {
                        Text("キャンセル")
                    }
                    Spacer(modifier = Modifier.width(8.dp))
                    Button(
                        onClick = {
                            val finalRule = when (ruleType) {
                                "domain" -> "||${ruleText.trim()}^"
                                "exception" -> "@@||${ruleText.trim()}"
                                else -> ruleText.trim()
                            }
                            if (finalRule.isNotEmpty()) {
                                onAdd(finalRule)
                            }
                        },
                        enabled = ruleText.isNotBlank()
                    ) {
                        Text("追加")
                    }
                }
            }
        }
    }
}

@Composable
fun ImportRulesDialog(
    onDismiss: () -> Unit,
    onImport: (List<String>) -> Unit
) {
    var rulesText by remember { mutableStateOf("") }
    
    Dialog(onDismissRequest = onDismiss) {
        Card(
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(16.dp)
        ) {
            Column(
                modifier = Modifier.padding(24.dp)
            ) {
                Text(
                    "フィルタールールをインポート",
                    style = MaterialTheme.typography.headlineSmall
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                Text(
                    "複数のルールを改行で区切って入力してください:",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                
                Spacer(modifier = Modifier.height(8.dp))
                
                OutlinedTextField(
                    value = rulesText,
                    onValueChange = { rulesText = it },
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(200.dp),
                    placeholder = {
                        Text(
                            "||example.com^\n||ads.example.com^\n@@||allowed.com",
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                )
                
                Spacer(modifier = Modifier.height(24.dp))
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End
                ) {
                    TextButton(onClick = onDismiss) {
                        Text("キャンセル")
                    }
                    Spacer(modifier = Modifier.width(8.dp))
                    Button(
                        onClick = {
                            val rules = rulesText
                                .split("\n")
                                .map { it.trim() }
                                .filter { it.isNotEmpty() }
                            if (rules.isNotEmpty()) {
                                onImport(rules)
                            }
                        },
                        enabled = rulesText.isNotBlank()
                    ) {
                        Text("インポート")
                    }
                }
            }
        }
    }
}

@Composable
fun RuleDetailDialog(
    rule: String,
    onDismiss: () -> Unit,
    onDelete: () -> Unit
) {
    Dialog(onDismissRequest = onDismiss) {
        Card(
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(16.dp)
        ) {
            Column(
                modifier = Modifier.padding(24.dp)
            ) {
                Text(
                    "フィルタールールの詳細",
                    style = MaterialTheme.typography.headlineSmall
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                // Rule display
                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    color = MaterialTheme.colorScheme.surfaceVariant,
                    shape = RoundedCornerShape(8.dp)
                ) {
                    Text(
                        text = rule,
                        modifier = Modifier.padding(12.dp),
                        style = MaterialTheme.typography.bodyMedium,
                        fontFamily = FontFamily.Monospace
                    )
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                // Rule explanation
                val explanation = when {
                    rule.startsWith("||") && rule.endsWith("^") -> {
                        val domain = rule.substring(2, rule.length - 1)
                        "このルールは「$domain」ドメインからのすべてのリクエストをブロックします。"
                    }
                    rule.startsWith("@@") -> {
                        val pattern = rule.substring(2)
                        "このルールは「$pattern」にマッチするリクエストを例外として許可します。"
                    }
                    rule.startsWith("##") -> {
                        "このルールはページ内の特定の要素を非表示にします。"
                    }
                    rule.contains("*") -> {
                        "このルールはワイルドカード（*）を使用してパターンマッチングを行います。"
                    }
                    else -> {
                        "カスタムフィルタールールです。"
                    }
                }
                
                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.secondaryContainer
                    )
                ) {
                    Row(
                        modifier = Modifier.padding(12.dp),
                        verticalAlignment = Alignment.Top
                    ) {
                        Icon(
                            Icons.Default.Info,
                            contentDescription = null,
                            modifier = Modifier.size(20.dp),
                            tint = MaterialTheme.colorScheme.onSecondaryContainer
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = explanation,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSecondaryContainer
                        )
                    }
                }
                
                Spacer(modifier = Modifier.height(24.dp))
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    TextButton(
                        onClick = onDelete,
                        colors = ButtonDefaults.textButtonColors(
                            contentColor = MaterialTheme.colorScheme.error
                        )
                    ) {
                        Icon(Icons.Default.Delete, contentDescription = null)
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("削除")
                    }
                    
                    Button(onClick = onDismiss) {
                        Text("閉じる")
                    }
                }
            }
        }
    }
}