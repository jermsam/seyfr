package com.jitpomi.seyfr

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.seyfr_core.Core

sealed class TransferStatus {
    data object Idle : TransferStatus()
    data object Sending : TransferStatus()
    data object Receiving : TransferStatus()
    data class Success(val message: String) : TransferStatus()
    data class Error(val message: String) : TransferStatus()
}

data class AppUiState(
    val ticket: String = "",
    val sendStatus: TransferStatus = TransferStatus.Idle,
    val receiveStatus: TransferStatus = TransferStatus.Idle,
    val selectedFileName: String? = null,
    val destinationPath: String = ""
)

class AppViewModel(context: Context) : ViewModel() {
    private val _uiState = MutableStateFlow(AppUiState())
    val uiState: StateFlow<AppUiState> = _uiState.asStateFlow()

    private val core: Core

    init {
        Log.d("AppViewModel", "Initializing AppViewModel")
        
        // Load the native library
        try {
            System.loadLibrary("seyfr_core")
            Log.d("AppViewModel", "Native library loaded successfully")
        } catch (e: Exception) {
            Log.e("AppViewModel", "Failed to load native library", e)
        }
        
        val dataDir = context.filesDir.resolve("seyfr").apply { mkdirs() }
        Log.d("AppViewModel", "Data dir: ${dataDir.absolutePath}")
        
        try {
            core = Core(dataDir.absolutePath)
            Log.d("AppViewModel", "Core initialized successfully")
        } catch (e: Exception) {
            Log.e("AppViewModel", "Failed to initialize Core", e)
            throw RuntimeException("Failed to initialize Seyfr core: ${e.message}", e)
        }
        
        val defaultDest = context.getExternalFilesDir(null)?.resolve("received")?.apply { mkdirs() }
        _uiState.value = _uiState.value.copy(destinationPath = defaultDest?.absolutePath ?: "")
        Log.d("AppViewModel", "AppViewModel initialized successfully")
    }

    fun send(path: String) {
        val fileName = path.substringAfterLast('/')
        _uiState.value = _uiState.value.copy(
            selectedFileName = fileName,
            sendStatus = TransferStatus.Sending
        )

        viewModelScope.launch {
            try {
                val ticket = withContext(Dispatchers.IO) {
                    core.send(path, null)
                }
                _uiState.value = _uiState.value.copy(
                    ticket = ticket,
                    sendStatus = TransferStatus.Success("Ready to share")
                )
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    sendStatus = TransferStatus.Error(e.message ?: "Send failed")
                )
            }
        }
    }

    fun receive(ticket: String) {
        if (ticket.isBlank()) {
            _uiState.value = _uiState.value.copy(
                receiveStatus = TransferStatus.Error("Ticket is empty")
            )
            return
        }

        _uiState.value = _uiState.value.copy(receiveStatus = TransferStatus.Receiving)

        viewModelScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    core.receive(ticket, _uiState.value.destinationPath, null)
                }
                _uiState.value = _uiState.value.copy(
                    receiveStatus = TransferStatus.Success("Received successfully")
                )
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    receiveStatus = TransferStatus.Error(e.message ?: "Receive failed")
                )
            }
        }
    }

    fun clearSend() {
        _uiState.value = _uiState.value.copy(
            ticket = "",
            selectedFileName = null,
            sendStatus = TransferStatus.Idle
        )
    }

    fun setDestination(path: String) {
        _uiState.value = _uiState.value.copy(destinationPath = path)
    }
}
