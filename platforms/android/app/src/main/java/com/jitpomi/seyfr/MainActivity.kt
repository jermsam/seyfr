package com.jitpomi.seyfr

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.KeyboardArrowUp
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.FavoriteBorder
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.jitpomi.seyfr.ui.components.AppLogo
import com.jitpomi.seyfr.ui.screens.ReceiveScreen
import com.jitpomi.seyfr.ui.screens.SendScreen
import com.jitpomi.seyfr.ui.screens.SupportScreen
import com.jitpomi.seyfr.ui.theme.SeyfrTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Initialize ndk-context before any Rust code that needs it
        // (e.g. hickory-resolver DNS via iroh::Endpoint::bind)
        JffiAndroidInit.initNdkContext(applicationContext)

        enableEdgeToEdge()
        setContent {
            SeyfrTheme {
                val viewModel = remember { AppViewModel(applicationContext) }
                val uiState by viewModel.uiState.collectAsStateWithLifecycle()

                SeyfrApp(
                    uiState = uiState,
                    onSend = viewModel::send,
                    onReceive = viewModel::receive,
                    onClearSend = viewModel::clearSend,
                    onSetDestination = viewModel::setDestination
                )
            }
        }
    }
}

@Composable
fun SeyfrApp(
    uiState: AppUiState,
    onSend: (String) -> Unit,
    onReceive: (String) -> Unit,
    onClearSend: () -> Unit,
    onSetDestination: (String) -> Unit
) {
    var selectedTab by remember { mutableStateOf(0) }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        bottomBar = {
            NavigationBar(
                containerColor = MaterialTheme.colorScheme.surface,
                tonalElevation = 0.dp
            ) {
                NavigationBarItem(
                    selected = selectedTab == 0,
                    onClick = { selectedTab = 0 },
                    icon = {
                        Icon(
                            imageVector = Icons.Outlined.KeyboardArrowUp,
                            contentDescription = "Send"
                        )
                    },
                    label = { Text("Send") }
                )
                NavigationBarItem(
                    selected = selectedTab == 1,
                    onClick = { selectedTab = 1 },
                    icon = {
                        Icon(
                            imageVector = Icons.Outlined.KeyboardArrowDown,
                            contentDescription = "Receive"
                        )
                    },
                    label = { Text("Receive") }
                )
                NavigationBarItem(
                    selected = selectedTab == 2,
                    onClick = { selectedTab = 2 },
                    icon = {
                        Icon(
                            imageVector = Icons.Outlined.FavoriteBorder,
                            contentDescription = "Support"
                        )
                    },
                    label = { Text("Support") }
                )
            }
        }
    ) { paddingValues ->
        Surface(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
            color = MaterialTheme.colorScheme.background
        ) {
            Column {
                AppLogo()

                when (selectedTab) {
                    0 -> SendScreen(
                        uiState = uiState,
                        onSend = onSend,
                        onClearSend = onClearSend
                    )
                    1 -> ReceiveScreen(
                        uiState = uiState,
                        onReceive = onReceive,
                        onSetDestination = onSetDestination
                    )
                    2 -> SupportScreen()
                }
            }
        }
    }
}
