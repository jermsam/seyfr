package com.example.seyfr.ui.screens

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.seyfr.AppUiState
import com.example.seyfr.TransferStatus
import com.example.seyfr.ui.components.FileRings
import com.example.seyfr.ui.components.FolderRings
import com.example.seyfr.ui.components.PrimaryButton
import com.example.seyfr.ui.components.QRCodeView
import com.example.seyfr.ui.components.SecondaryButton

@Composable
fun SendScreen(
    uiState: AppUiState,
    onSend: (String) -> Unit,
    onClearSend: () -> Unit,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    var isFolderMode by remember { mutableStateOf(false) }

    val filePicker = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri ->
        uri?.let {
            val path = getRealPathFromURI(context, it)
            path?.let(onSend)
        }
    }

    val folderPicker = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.OpenDocumentTree()
    ) { uri ->
        uri?.let {
            // Persist permission and resolve to real path
            context.contentResolver.takePersistableUriPermission(
                it,
                android.content.Intent.FLAG_GRANT_READ_URI_PERMISSION
            )
            val path = getFolderPathFromURI(context, it)
            path?.let(onSend)
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(vertical = 20.dp),
        verticalArrangement = Arrangement.spacedBy(32.dp)
    ) {
        if (uiState.sendStatus is TransferStatus.Idle && uiState.selectedFileName == null) {
            Column(
                modifier = Modifier.padding(horizontal = 20.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Box(
                    modifier = Modifier.clickable {
                        if (isFolderMode) {
                            folderPicker.launch(null)
                        } else {
                            filePicker.launch("*/*")
                        }
                    }
                ) {
                    if (isFolderMode) {
                        FolderRings()
                    } else {
                        FileRings()
                    }
                }

                Spacer(modifier = Modifier.height(16.dp))

                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Text(
                        text = "File mode",
                        fontSize = 13.sp,
                        fontWeight = if (!isFolderMode) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (!isFolderMode) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Switch(
                        checked = isFolderMode,
                        onCheckedChange = { isFolderMode = it },
                        colors = SwitchDefaults.colors(
                            checkedThumbColor = MaterialTheme.colorScheme.onSurface,
                            checkedTrackColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.5f)
                        )
                    )
                    Text(
                        text = "Folder mode",
                        fontSize = 13.sp,
                        fontWeight = if (isFolderMode) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (isFolderMode) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }

        AnimatedVisibility(
            visible = uiState.selectedFileName != null,
            enter = slideInVertically() + fadeIn(),
            exit = slideOutVertically() + fadeOut()
        ) {
            Column(
                modifier = Modifier.padding(horizontal = 20.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Text(
                    text = "Active Transfers",
                    fontSize = 17.sp,
                    fontWeight = FontWeight.SemiBold
                )

                FileCard(
                    fileName = uiState.selectedFileName ?: "",
                    isLoading = uiState.sendStatus is TransferStatus.Sending
                )
            }
        }

        AnimatedVisibility(
            visible = uiState.ticket.isNotEmpty(),
            enter = slideInVertically() + fadeIn(),
            exit = slideOutVertically() + fadeOut()
        ) {
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
                            text = "Transfer Ticket",
                            fontSize = 15.sp,
                            fontWeight = FontWeight.SemiBold
                        )

                        IconButton(onClick = onClearSend) {
                            Icon(
                                imageVector = Icons.Default.Close,
                                contentDescription = "Clear",
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }

                    QRCodeView(ticket = uiState.ticket)

                    Surface(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp),
                        border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline)
                    ) {
                        Text(
                            text = uiState.ticket,
                            modifier = Modifier.padding(14.dp),
                            fontSize = 12.sp,
                            fontFamily = FontFamily.Monospace,
                            lineHeight = 18.sp
                        )
                    }

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Box(modifier = Modifier.weight(1f)) {
                            SecondaryButton(
                                title = "Copy",
                                icon = Icons.Outlined.ContentCopy,
                                onClick = {
                                    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                                    clipboard.setPrimaryClip(ClipData.newPlainText("ticket", uiState.ticket))
                                }
                            )
                        }

                        Box(modifier = Modifier.weight(1f)) {
                            PrimaryButton(
                                title = "Share",
                                icon = Icons.Outlined.Share,
                                onClick = {
                                    val intent = Intent(Intent.ACTION_SEND).apply {
                                        type = "text/plain"
                                        putExtra(Intent.EXTRA_TEXT, uiState.ticket)
                                    }
                                    context.startActivity(Intent.createChooser(intent, "Share ticket"))
                                }
                            )
                        }
                    }
                }
            }
        }

        StatusPill(status = uiState.sendStatus)
    }
}

@Composable
private fun FileCard(fileName: String, isLoading: Boolean) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = fileName,
                modifier = Modifier.weight(1f),
                fontSize = 14.sp
            )
            if (isLoading) {
                Text(
                    text = "Sending...",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            } else {
                Text(
                    text = "✓ Completed",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.primary
                )
            }
        }
    }
}

@Composable
private fun StatusPill(status: TransferStatus) {
    AnimatedVisibility(
        visible = status is TransferStatus.Success || status is TransferStatus.Error,
        enter = fadeIn(),
        exit = fadeOut()
    ) {
        val (text, color) = when (status) {
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

private fun getRealPathFromURI(context: Context, uri: android.net.Uri): String? {
    return try {
        val fileName = context.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) {
                val index = cursor.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                if (index >= 0) cursor.getString(index) else null
            } else null
        } ?: uri.lastPathSegment ?: "unknown"

        // Copy content URI to app-private file so Rust can access it
        val destFile = java.io.File(context.cacheDir, fileName)
        context.contentResolver.openInputStream(uri)?.use { input ->
            destFile.outputStream().use { output ->
                input.copyTo(output)
            }
        }
        destFile.absolutePath
    } catch (e: Exception) {
        android.util.Log.e("SendScreen", "Failed to copy file from URI", e)
        null
    }
}

private fun getFolderPathFromURI(context: Context, treeUri: android.net.Uri): String? {
    return try {
        val destDir = java.io.File(context.cacheDir, "picked_folder_${System.currentTimeMillis()}")
        destDir.mkdirs()

        val docUri = android.provider.DocumentsContract.buildDocumentUriUsingTree(
            treeUri,
            android.provider.DocumentsContract.getTreeDocumentId(treeUri)
        )
        copyDocumentTree(context, docUri, destDir)
        destDir.absolutePath
    } catch (e: Exception) {
        android.util.Log.e("SendScreen", "Failed to copy folder from URI", e)
        null
    }
}

private fun copyDocumentTree(context: Context, docUri: android.net.Uri, destDir: java.io.File) {
    if (android.provider.DocumentsContract.isDocumentUri(context, docUri)) {
        val mimeType = context.contentResolver.getType(docUri)
        if (mimeType == null || mimeType == android.provider.DocumentsContract.Document.MIME_TYPE_DIR) {
            // It's a directory - recurse
            val childrenUri = android.provider.DocumentsContract.buildChildDocumentsUriUsingTree(
                docUri,
                android.provider.DocumentsContract.getDocumentId(docUri)
            )
            context.contentResolver.query(childrenUri, null, null, null, null)?.use { cursor ->
                while (cursor.moveToNext()) {
                    val childDocId = cursor.getString(
                        cursor.getColumnIndexOrThrow(android.provider.DocumentsContract.Document.COLUMN_DOCUMENT_ID)
                    )
                    val childUri = android.provider.DocumentsContract.buildDocumentUriUsingTree(docUri, childDocId)
                    copyDocumentTree(context, childUri, destDir)
                }
            }
        } else {
            // It's a file - copy it
            val name = context.contentResolver.query(docUri, null, null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    cursor.getString(cursor.getColumnIndexOrThrow(android.provider.DocumentsContract.Document.COLUMN_DISPLAY_NAME))
                } else null
            } ?: docUri.lastPathSegment ?: "file"

            val destFile = java.io.File(destDir, name)
            context.contentResolver.openInputStream(docUri)?.use { input ->
                destFile.outputStream().use { output ->
                    input.copyTo(output)
                }
            }
        }
    }
}
