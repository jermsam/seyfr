package com.jitpomi.seyfr.ui.screens

import android.widget.Toast
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
import androidx.compose.material.icons.outlined.AccountBalance
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.FavoriteBorder
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material.icons.outlined.Public
import androidx.compose.material.icons.outlined.QrCode
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Tab
import androidx.compose.material3.TabRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.jitpomi.seyfr.ui.components.PrimaryButton
import com.jitpomi.seyfr.ui.components.QRCodeView

@Composable
fun SupportScreen(modifier: Modifier = Modifier) {
    val context = LocalContext.current
    val clipboardManager = LocalClipboardManager.current
    val uriHandler = LocalUriHandler.current
    val scrollState = rememberScrollState()
    var selectedSubTab by remember { mutableStateOf(0) }
    var selectedBankTab by remember { mutableStateOf(0) }

    fun copyToClipboard(label: String, text: String) {
        clipboardManager.setText(AnnotatedString(text))
        Toast.makeText(context, "$label copied to clipboard", Toast.LENGTH_SHORT).show()
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(scrollState)
            .padding(vertical = 20.dp),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        // Header Card
        Column(
            modifier = Modifier.padding(horizontal = 20.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Surface(
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.primary.copy(alpha = 0.15f),
                modifier = Modifier.padding(bottom = 4.dp)
            ) {
                Icon(
                    imageVector = Icons.Outlined.FavoriteBorder,
                    contentDescription = "Support",
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.padding(16.dp)
                )
            }

            Text(
                text = "Support JITPOMI",
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = "Seyfr is created by JITPOMI LLC. Your support helps keep Seyfr AD-free, fast, and secure.",
                fontSize = 14.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = 16.dp),
                textAlign = TextAlign.Center
            )
        }

        // Sub Tabs for Venmo / Bank
        TabRow(
            selectedTabIndex = selectedSubTab,
            containerColor = MaterialTheme.colorScheme.background,
            contentColor = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(horizontal = 20.dp)
        ) {
            Tab(
                selected = selectedSubTab == 0,
                onClick = { selectedSubTab = 0 },
                text = {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Icon(Icons.Outlined.QrCode, contentDescription = "Venmo QR")
                        Text("Venmo QR")
                    }
                }
            )
            Tab(
                selected = selectedSubTab == 1,
                onClick = { selectedSubTab = 1 },
                text = {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Icon(Icons.Outlined.AccountBalance, contentDescription = "Bank Transfer")
                        Text("Bank Transfer")
                    }
                }
            )
        }

        when (selectedSubTab) {
            0 -> {
                // Venmo Card
                Card(
                    modifier = Modifier.padding(horizontal = 20.dp),
                    shape = RoundedCornerShape(20.dp),
                    border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
                ) {
                    Column(
                        modifier = Modifier.padding(24.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Text(
                            text = "Scan to Support via Venmo",
                            fontSize = 16.sp,
                            fontWeight = FontWeight.SemiBold
                        )

                        Surface(
                            shape = RoundedCornerShape(16.dp),
                            color = androidx.compose.ui.graphics.Color.White,
                            modifier = Modifier.padding(vertical = 8.dp)
                        ) {
                            QRCodeView(
                                ticket = "https://venmo.com/u/jitpomi",
                                modifier = Modifier.padding(16.dp)
                            )
                        }

                        Text(
                            text = "@jitpomi",
                            fontSize = 18.sp,
                            fontFamily = FontFamily.Monospace,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.clickable {
                                copyToClipboard("Venmo Handle", "@jitpomi")
                            }
                        )

                        PrimaryButton(
                            title = "Open Venmo App",
                            icon = Icons.AutoMirrored.Outlined.OpenInNew,
                            onClick = { uriHandler.openUri("https://venmo.com/u/jitpomi") }
                        )
                    }
                }
            }

            1 -> {
                // Bank Details Sub-Navigation
                TabRow(
                    selectedTabIndex = selectedBankTab,
                    containerColor = MaterialTheme.colorScheme.background,
                    contentColor = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.padding(horizontal = 20.dp)
                ) {
                    Tab(
                        selected = selectedBankTab == 0,
                        onClick = { selectedBankTab = 0 },
                        text = { Text("Domestic (USD)", fontSize = 12.sp) }
                    )
                    Tab(
                        selected = selectedBankTab == 1,
                        onClick = { selectedBankTab = 1 },
                        text = { Text("Intl (USD)", fontSize = 12.sp) }
                    )
                    Tab(
                        selected = selectedBankTab == 2,
                        onClick = { selectedBankTab = 2 },
                        text = { Text("Intl (FX / Non-USD)", fontSize = 12.sp) }
                    )
                }

                Card(
                    modifier = Modifier.padding(horizontal = 20.dp),
                    shape = RoundedCornerShape(20.dp),
                    border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
                ) {
                    Column(
                        modifier = Modifier.padding(24.dp),
                        verticalArrangement = Arrangement.spacedBy(20.dp)
                    ) {
                        when (selectedBankTab) {
                            0 -> {
                                Text(
                                    text = "Domestic ACH & Wire (USD)",
                                    fontSize = 16.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    modifier = Modifier.padding(bottom = 4.dp)
                                )

                                BankDetailRow(
                                    label = "Beneficiary Name",
                                    value = "JITPOMI LLC",
                                    onCopy = { copyToClipboard("Beneficiary Name", "JITPOMI LLC") }
                                )

                                BankDetailRow(
                                    label = "Account Number",
                                    value = "202617088912",
                                    onCopy = { copyToClipboard("Account Number", "202617088912") }
                                )

                                BankDetailRow(
                                    label = "Routing Number (ABA / ACH)",
                                    value = "091311229",
                                    onCopy = { copyToClipboard("Routing Number", "091311229") }
                                )

                                BankDetailRow(
                                    label = "Bank Name",
                                    value = "Choice Financial Group (Mercury Partner)",
                                    onCopy = { copyToClipboard("Bank Name", "Choice Financial Group") }
                                )

                                BankDetailRow(
                                    label = "Bank Address",
                                    value = "4501 23rd Avenue S, Fargo, ND 58104 US",
                                    onCopy = { copyToClipboard("Bank Address", "4501 23rd Avenue S, Fargo, ND 58104 US") }
                                )

                                BankDetailRow(
                                    label = "Beneficiary Address",
                                    value = "5003 59th Avenue Court West, Tacoma, WA 98467 US",
                                    onCopy = { copyToClipboard("Beneficiary Address", "5003 59th Avenue Court West, Tacoma, WA 98467 US") }
                                )
                            }

                            1 -> {
                                Text(
                                    text = "International Wire (USD)",
                                    fontSize = 16.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    modifier = Modifier.padding(bottom = 4.dp)
                                )

                                BankDetailRow(
                                    label = "SWIFT / BIC Code",
                                    value = "CHFGUS44021",
                                    onCopy = { copyToClipboard("SWIFT Code", "CHFGUS44021") }
                                )

                                BankDetailRow(
                                    label = "ABA Routing Number",
                                    value = "091311229",
                                    onCopy = { copyToClipboard("Routing Number", "091311229") }
                                )

                                BankDetailRow(
                                    label = "IBAN / Account Number",
                                    value = "202617088912",
                                    onCopy = { copyToClipboard("Account Number", "202617088912") }
                                )

                                BankDetailRow(
                                    label = "Beneficiary Name",
                                    value = "JITPOMI LLC",
                                    onCopy = { copyToClipboard("Beneficiary Name", "JITPOMI LLC") }
                                )

                                BankDetailRow(
                                    label = "Bank Name",
                                    value = "Choice Financial Group",
                                    onCopy = { copyToClipboard("Bank Name", "Choice Financial Group") }
                                )

                                BankDetailRow(
                                    label = "Bank & Beneficiary Address",
                                    value = "Tacoma, WA / Fargo, ND USA",
                                    onCopy = { copyToClipboard("Address", "5003 59th Avenue Court West, Tacoma, WA 98467 USA") }
                                )
                            }

                            2 -> {
                                Text(
                                    text = "International Wire (FX / Foreign Currency)",
                                    fontSize = 16.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    modifier = Modifier.padding(bottom = 4.dp)
                                )

                                Surface(
                                    shape = RoundedCornerShape(12.dp),
                                    color = MaterialTheme.colorScheme.primary.copy(alpha = 0.1f),
                                    modifier = Modifier.fillMaxWidth()
                                ) {
                                    Column(
                                        modifier = Modifier.padding(16.dp),
                                        verticalArrangement = Arrangement.spacedBy(8.dp)
                                    ) {
                                        Row(
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                                        ) {
                                            Icon(
                                                imageVector = Icons.Outlined.Info,
                                                contentDescription = "Important",
                                                tint = MaterialTheme.colorScheme.primary
                                            )
                                            Text(
                                                text = "Required Payment Reference / Memo:",
                                                fontSize = 13.sp,
                                                fontWeight = FontWeight.Bold,
                                                color = MaterialTheme.colorScheme.primary
                                            )
                                        }

                                        Text(
                                            text = "/FFC/202617088912/JITPOMI LLC/Tacoma, USA",
                                            fontSize = 13.sp,
                                            fontFamily = FontFamily.Monospace,
                                            fontWeight = FontWeight.Bold,
                                            modifier = Modifier
                                                .clickable { copyToClipboard("Payment Memo", "/FFC/202617088912/JITPOMI LLC/Tacoma, USA") }
                                                .padding(vertical = 4.dp)
                                        )

                                        Text(
                                            text = "You must include this exact string in the wire Memo / Reference field.",
                                            fontSize = 11.sp,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                }

                                BankDetailRow(
                                    label = "Intermediary SWIFT / BIC Code",
                                    value = "CHASUS33XXX",
                                    onCopy = { copyToClipboard("Intermediary SWIFT", "CHASUS33XXX") }
                                )

                                BankDetailRow(
                                    label = "Intermediary ABA Routing",
                                    value = "021000021",
                                    onCopy = { copyToClipboard("Intermediary Routing", "021000021") }
                                )

                                BankDetailRow(
                                    label = "Intermediary Bank Name",
                                    value = "JP Morgan Chase Bank, N.A. – New York",
                                    onCopy = { copyToClipboard("Bank Name", "JP Morgan Chase Bank, N.A. – New York") }
                                )

                                BankDetailRow(
                                    label = "IBAN / Account Number",
                                    value = "707567692",
                                    onCopy = { copyToClipboard("Account Number", "707567692") }
                                )

                                BankDetailRow(
                                    label = "Beneficiary Name",
                                    value = "Choice Financial Group",
                                    onCopy = { copyToClipboard("Beneficiary Name", "Choice Financial Group") }
                                )
                            }
                        }

                        Spacer(modifier = Modifier.height(4.dp))
                    }
                }
            }
        }
    }
}

@Composable
private fun BankDetailRow(
    label: String,
    value: String,
    onCopy: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = label,
                fontSize = 12.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Text(
                text = value,
                fontSize = 15.sp,
                fontWeight = FontWeight.Medium,
                fontFamily = if (value.any { it.isDigit() }) FontFamily.Monospace else FontFamily.Default,
                modifier = Modifier.padding(top = 2.dp)
            )
        }

        IconButton(onClick = onCopy) {
            Icon(
                imageVector = Icons.Outlined.ContentCopy,
                contentDescription = "Copy $label",
                tint = MaterialTheme.colorScheme.primary
            )
        }
    }
}
