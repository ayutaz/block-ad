package com.adblock

import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.adblock.ui.theme.AdBlockTheme
import com.adblock.vpn.AdBlockVpnService

class MainActivity : ComponentActivity() {
    
    private val vpnPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == RESULT_OK) {
            startVpnService()
        }
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
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
}

@Composable
fun MainScreen(onToggleVpn: (Boolean) -> Unit) {
    var isVpnEnabled by remember { mutableStateOf(false) }
    
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = "AdBlock",
            style = MaterialTheme.typography.headlineLarge
        )
        
        Spacer(modifier = Modifier.height(32.dp))
        
        Switch(
            checked = isVpnEnabled,
            onCheckedChange = { enabled ->
                isVpnEnabled = enabled
                onToggleVpn(enabled)
            }
        )
        
        Text(
            text = if (isVpnEnabled) "Ad blocking is ON" else "Ad blocking is OFF",
            style = MaterialTheme.typography.bodyLarge,
            modifier = Modifier.padding(top = 16.dp)
        )
    }
}