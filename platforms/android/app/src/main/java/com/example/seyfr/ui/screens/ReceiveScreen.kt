package com.example.seyfr.ui.screens

import android.Manifest
import android.content.pm.PackageManager
import android.net.Uri
import android.provider.DocumentsContract
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentPaste
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import com.example.seyfr.AppUiState
import com.example.seyfr.TransferStatus
import com.example.seyfr.ui.components.PrimaryButton
import com.example.seyfr.ui.components.QRRings
import com.example.seyfr.ui.components.QRScanner

@Composable
fun ReceiveScreen(
    uiState: AppUiState,
    onReceive: (String) -> Unit,
    onSetDestination: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    val clipboardManager = LocalClipboardManager.current
    var ticketInput by remember { mutableStateOf("") }
    var showQRScanner by remember { mutableStateOf(false) }

    val cameraPermissionLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission()
    ) { isGranted ->
        if (isGranted) {
            showQRScanner = true
        }
    }

    val folderPickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.OpenDocumentTree()
    ) { uri ->
        uri?.let {
            val path = resolveTreeUriToPath(context, it)
            onSetDestination(path)
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(vertical = 20.dp),
        verticalArrangement = Arrangement.spacedBy(28.dp)
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 20.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            QRRings(
                modifier = Modifier.clickable {
                    when (PackageManager.PERMISSION_GRANTED) {
                        ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) -> {
                            showQRScanner = true
                        }
                        else -> {
                            cameraPermissionLauncher.launch(Manifest.permission.CAMERA)
                        }
                    }
                }
            )

            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                Text(
                    text = "Receive a file",
                    fontSize = 17.sp,
                    fontWeight = FontWeight.SemiBold
                )
                Text(
                    text = "Tap to scan a QR code or paste below",
                    fontSize = 13.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }

        Card(
            modifier = Modifier.padding(horizontal = 20.dp),
            shape = RoundedCornerShape(20.dp),
            border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
        ) {
            Column(
                modifier = Modifier.padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "Ticket",
                        fontSize = 15.sp,
                        fontWeight = FontWeight.SemiBold
                    )

                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        TextButton(
                            onClick = {
                                clipboardManager.getText()?.text?.let {
                                    ticketInput = it.toString()
                                }
                            }
                        ) {
                            Icon(
                                imageVector = Icons.Outlined.ContentPaste,
                                contentDescription = "Paste"
                            )
                            Text("Paste", modifier = Modifier.padding(start = 4.dp))
                        }

                        if (ticketInput.isNotEmpty()) {
                            TextButton(onClick = { ticketInput = "" }) {
                                Text("Clear")
                            }
                        }
                    }
                }

                OutlinedTextField(
                    value = ticketInput,
                    onValueChange = { ticketInput = it },
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(120.dp),
                    placeholder = { Text("Paste ticket here...") },
                    shape = RoundedCornerShape(12.dp),
                    colors = OutlinedTextFieldDefaults.colors(
                        unfocusedBorderColor = MaterialTheme.colorScheme.outline.copy(alpha = 0.5f)
                    )
                )
            }
        }

        Card(
            modifier = Modifier.padding(horizontal = 20.dp),
            shape = RoundedCornerShape(20.dp),
            border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
        ) {
            Column(
                modifier = Modifier.padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp)
            ) {
                Text(
                    text = "Save Location",
                    fontSize = 15.sp,
                    fontWeight = FontWeight.SemiBold
                )

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Outlined.Folder,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Column {
                            Text(
                                text = "Documents",
                                fontSize = 14.sp,
                                fontWeight = FontWeight.Medium
                            )
                            Text(
                                text = uiState.destinationPath.takeLast(30),
                                fontSize = 12.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }

                    TextButton(onClick = { folderPickerLauncher.launch(null) }) {
                        Text("Change")
                    }
                }
            }
        }

        PrimaryButton(
            title = "Receive File",
            icon = Icons.Outlined.Download,
            onClick = { onReceive(ticketInput) },
            modifier = Modifier.padding(horizontal = 20.dp)
        )

        AnimatedVisibility(
            visible = uiState.receiveStatus is TransferStatus.Success || uiState.receiveStatus is TransferStatus.Error,
            enter = fadeIn(),
            exit = fadeOut()
        ) {
            val (text, color) = when (val status = uiState.receiveStatus) {
                is TransferStatus.Success -> status.message to MaterialTheme.colorScheme.primary
                is TransferStatus.Error -> status.message to MaterialTheme.colorScheme.error
                else -> "" to MaterialTheme.colorScheme.onSurface
            }

            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp),
                shape = RoundedCornerShape(12.dp),
                color = color.copy(alpha = 0.1f)
            ) {
                Text(
                    text = text,
                    modifier = Modifier.padding(12.dp),
                    fontSize = 13.sp,
                    color = color
                )
            }
        }
    }

    if (showQRScanner) {
        QRScannerDialog(
            onDismiss = { showQRScanner = false },
            onScan = { scannedText ->
                ticketInput = scannedText
                showQRScanner = false
            }
        )
    }
}

/**
 * Resolve a Storage Access Framework (SAF) tree URI to a real filesystem path.
 *
 * This handles common document providers (Downloads, ExternalStorage) and falls
 * back to creating a directory in app-private external storage when the URI
 * cannot be directly mapped to a path (e.g., Google Drive, cloud providers).
 */
private fun resolveTreeUriToPath(context: android.content.Context, treeUri: Uri): String {
    val docId = DocumentsContract.getTreeDocumentId(treeUri)

    // Handle primary external storage (e.g., content://com.android.externalstorage.documents/tree/primary:Download)
    if (docId.startsWith("primary:")) {
        val relativePath = docId.removePrefix("primary:")
        val base = android.os.Environment.getExternalStorageDirectory()
        return java.io.File(base, relativePath).absolutePath
    }

    // Try to extract from content URI path segments
    val path = treeUri.path
    if (path != null) {
        // Some providers encode the real path after /tree/ or /document/
        val realPath = path
            .replace("/tree/", "")
            .replace("/document/", "")
            .replaceFirst("primary:", "")
        if (!realPath.startsWith("content:") && realPath.isNotBlank()) {
            val base = android.os.Environment.getExternalStorageDirectory()
            val candidate = java.io.File(base, realPath)
            if (candidate.exists() || candidate.parentFile?.exists() == true) {
                return candidate.absolutePath
            }
        }
    }

    // Fallback: create a named directory in app-private external storage
    val fallbackDir = context.getExternalFilesDir(null)
        ?.resolve("received")
        ?.apply { mkdirs() }
    return fallbackDir?.absolutePath
        ?: context.filesDir.resolve("received").apply { mkdirs() }.absolutePath
}

@Composable
private fun QRScannerDialog(
    onDismiss: () -> Unit,
    onScan: (String) -> Unit
) {
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(
            usePlatformDefaultWidth = false,
            dismissOnClickOutside = true,
            dismissOnBackPress = true
        )
    ) {
        QRScanner(
            onScan = { scannedText ->
                onScan(scannedText)
            },
            onDismiss = onDismiss
        )
    }
}
