// SonyBridge v0.2.0 UI Controller

const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : async (cmd, args) => {
  console.log(`[Mock Tauri Invoke] ${cmd}`, args);
  return {
    status: "ok",
    wire_hex: "3E 0C 01 00 00 00 08 68 02 01 01 02 01 00 00 8C 3C"
  };
};

const I18N = {
  en: {
    subtitle: "WH-1000XM3 / XM4 / XM5 Desktop Companion",
    tab_dashboard: "Dashboard & EQ",
    tab_audio: "Windows & Power",
    tab_architecture: "Architecture & Wire",
    status_connected: "Connected",
    btn_reconnect: "Reconnect",
    quick_profiles: "Quick Sound Profiles:",
    telemetry_title: "Device Telemetry",
    anc_title: "Noise Canceling & Ambient Sound",
    mode_nc: "Noise Canceling",
    mode_wind: "Wind Reduction",
    mode_ambient: "Ambient Sound",
    mode_off: "Off",
    ambient_level: "Ambient Sound Level",
    voice_focus: "Focus on Voice (attenuate background drone)",
    eq_title: "Hardware Equalizer",
    clear_bass: "Clear Bass (DSP Sub-Bass)",
    enforce_a2dp: "Enforce A2DP",
    autostart_label: "Launch SonyBridge on Windows Login"
  },
  it: {
    subtitle: "Assistente Desktop WH-1000XM3 / XM4 / XM5",
    tab_dashboard: "Pannello & EQ",
    tab_audio: "Windows & Energia",
    tab_architecture: "Architettura & Pacchetti",
    status_connected: "Connesso",
    btn_reconnect: "Riconnetti",
    quick_profiles: "Profili Audio Rapidi:",
    telemetry_title: "Telemetria Dispositivo",
    anc_title: "Eliminazione Rumore & Suono Ambiente",
    mode_nc: "Eliminazione Rumore",
    mode_wind: "Riduzione Vento",
    mode_ambient: "Suono Ambiente",
    mode_off: "Disattivato",
    ambient_level: "Livello Suono Ambiente",
    voice_focus: "Risalta Voce (attenua ronzio di fondo)",
    eq_title: "Equalizzatore Hardware",
    clear_bass: "Clear Bass (Sub-Bassi DSP)",
    enforce_a2dp: "Forza Stereo A2DP",
    autostart_label: "Avvia SonyBridge all'accesso di Windows"
  }
};

// Preset frequency maps
const PRESETS = {
  Off: { bands: [0, 0, 0, 0, 0], bass: 0 },
  Excited: { bands: [-1, 2, 4, 3, 1], bass: 3 },
  Bright: { bands: [-2, 0, 2, 4, 5], bass: 0 },
  Mellow: { bands: [2, 1, -1, -2, -3], bass: 1 },
  Relaxed: { bands: [1, 2, 0, -1, 0], bass: 2 },
  Vocal: { bands: [-2, 3, 5, 2, -1], bass: -1 },
  TrebleBoost: { bands: [-3, -1, 1, 4, 6], bass: 0 },
  BassBoost: { bands: [4, 2, 0, 0, 0], bass: 6 },
  Speech: { bands: [-4, 4, 6, 1, -2], bass: -4 },
  Custom1: { bands: [0, 0, 0, 0, 0], bass: 0 },
  Custom2: { bands: [0, 0, 0, 0, 0], bass: 0 }
};

let currentAncMode = "NoiseCanceling";
let debounceTimer = null;

function debounce(func, delay = 75) {
  return (...args) => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => func(...args), delay);
  };
}

let txCounter = 0;
function logMdrWireFrame(opcodeHex = "08", typeHex = "68") {
  txCounter = (txCounter + 1) & 0xFF;
  const seqHex = txCounter.toString(16).padStart(2, "0").toUpperCase();
  const hex = `3E 0C ${seqHex} 00 00 00 ${opcodeHex} ${typeHex} 02 01 01 02 01 00 00 8B 3C`;
  updateWireConsole({ wire_hex: hex });
}

function updateWireConsole(res) {
  if (res && res.wire_hex) {
    const el = document.getElementById("wire-console");
    if (el) el.textContent = res.wire_hex;
    const fine = document.getElementById("status-fine");
    if (fine) fine.textContent = `[TX #${txCounter}] ${res.wire_hex.slice(0, 32)}...`;
  }
}

function applyLanguage(lang) {
  const dict = I18N[lang] || I18N.en;
  document.querySelectorAll("[data-i18n]").forEach(el => {
    const key = el.getAttribute("data-i18n");
    if (dict[key]) {
      el.textContent = dict[key];
    }
  });
}

document.addEventListener("DOMContentLoaded", () => {
  // Navigation Tabs
  const navTabs = document.querySelectorAll(".nav-tab");
  const tabPanels = document.querySelectorAll(".tab-panel");
  navTabs.forEach(tab => {
    tab.addEventListener("click", () => {
      const wasActive = tab.classList.contains("active");
      navTabs.forEach(t => t.classList.remove("active"));
      tabPanels.forEach(p => p.classList.remove("active"));
      tab.classList.add("active");
      const target = document.getElementById(tab.dataset.tab);
      if (target) {
        target.classList.add("active");
        target.scrollTop = 0;
      }
      tab.style.opacity = "0.75";
      setTimeout(() => { tab.style.opacity = "1"; }, 80);
      logMdrWireFrame("03", wasActive ? "AA" : "01");
    });
  });

  // Language Selector
  const langSelect = document.getElementById("lang-select");
  langSelect.addEventListener("change", (e) => {
    const lang = e.target.value;
    applyLanguage(lang);
    invoke("set_language", { lang }).catch(console.error);
    logMdrWireFrame("01", "20");
  });

  // Onboarding Modal
  const onboardingModal = document.getElementById("onboarding-modal");
  const openWizardBtn = document.getElementById("btn-open-wizard");
  const closeWizardBtn = document.getElementById("btn-close-wizard");
  const finishWizardBtn = document.getElementById("btn-finish-onboarding");

  if (openWizardBtn && onboardingModal) {
    openWizardBtn.addEventListener("click", () => {
      onboardingModal.style.display = "flex";
      logMdrWireFrame("02", "4A");
    });
  }

  if (closeWizardBtn && onboardingModal) {
    closeWizardBtn.addEventListener("click", () => {
      onboardingModal.style.display = "none";
    });
  }

  if (finishWizardBtn && onboardingModal) {
    finishWizardBtn.addEventListener("click", () => {
      onboardingModal.style.display = "none";
      invoke("complete_onboarding").catch(console.error);
      const reconnectBtn = document.getElementById("btn-reconnect");
      if (reconnectBtn) reconnectBtn.click();
    });
  }

  // Dismiss modal on backdrop click or Escape key
  if (onboardingModal) {
    onboardingModal.addEventListener("click", (e) => {
      if (e.target === onboardingModal) {
        onboardingModal.style.display = "none";
      }
    });
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && onboardingModal.style.display === "flex") {
        onboardingModal.style.display = "none";
      }
    });
  }

  // Load saved AppConfig on startup
  invoke("get_app_config").then(cfg => {
    if (cfg && cfg.language) {
      langSelect.value = cfg.language;
      applyLanguage(cfg.language);
    }
    if (cfg && cfg.onboarding_completed === false) {
      onboardingModal.style.display = "flex";
    }
  }).catch(() => {});

  // ANC Mode Buttons
  const modeButtons = document.querySelectorAll(".seg-btn");
  const ambientGroup = document.getElementById("ambient-slider-group");
  const ambientSlider = document.getElementById("ambient-slider");
  const ambientVal = document.getElementById("ambient-level-val");
  const voiceFocusCheck = document.getElementById("voice-focus-check");

  function setUiAncMode(mode, level, voiceFocus) {
    currentAncMode = mode;
    modeButtons.forEach(b => {
      b.classList.toggle("active", b.dataset.mode === mode);
    });
    if (mode === "AmbientSound") {
      ambientGroup.style.opacity = "1";
      ambientGroup.style.pointerEvents = "auto";
      if (level) {
        ambientSlider.value = level;
        ambientVal.textContent = `Level ${level}`;
      }
      if (typeof voiceFocus === "boolean") {
        voiceFocusCheck.checked = voiceFocus;
      }
    } else {
      ambientGroup.style.opacity = "0.4";
      ambientGroup.style.pointerEvents = "none";
    }
  }

  modeButtons.forEach(btn => {
    btn.addEventListener("click", () => {
      setUiAncMode(btn.dataset.mode);
      sendAncUpdate();
    });
  });

  ambientSlider.addEventListener("input", (e) => {
    const val = e.target.value;
    ambientVal.textContent = `Level ${val}`;
    debounceSendAncUpdate();
  });

  voiceFocusCheck.addEventListener("change", () => {
    debounceSendAncUpdate();
  });

  const debounceSendAncUpdate = debounce(sendAncUpdate, 75);

  function sendAncUpdate() {
    const payload = {
      mode: currentAncMode,
      level: parseInt(ambientSlider.value, 10),
      focusOnVoice: voiceFocusCheck.checked
    };
    invoke("set_anc_mode", { payload }).then(updateWireConsole).catch(console.error);
  }

  // Equalizer Controls
  const presetSelect = document.getElementById("eq-preset-select");
  const eqBands = [
    { slider: document.getElementById("eq-400"), label: document.getElementById("val-400") },
    { slider: document.getElementById("eq-1k"), label: document.getElementById("val-1k") },
    { slider: document.getElementById("eq-2k5"), label: document.getElementById("val-2k5") },
    { slider: document.getElementById("eq-6k3"), label: document.getElementById("val-6k3") },
    { slider: document.getElementById("eq-16k"), label: document.getElementById("val-16k") }
  ];
  const clearBassSlider = document.getElementById("clear-bass-slider");
  const clearBassVal = document.getElementById("clear-bass-val");

  function updateEqLabels() {
    eqBands.forEach(b => {
      const v = parseInt(b.slider.value, 10);
      b.label.textContent = v > 0 ? `+${v}` : `${v}`;
    });
    const bass = parseInt(clearBassSlider.value, 10);
    clearBassVal.textContent = bass > 0 ? `+${bass}` : `${bass}`;
  }

  function applyUiPreset(presetName) {
    presetSelect.value = presetName;
    const preset = PRESETS[presetName];
    if (preset) {
      preset.bands.forEach((val, idx) => {
        eqBands[idx].slider.value = val;
      });
      clearBassSlider.value = preset.bass;
      updateEqLabels();
    }
  }

  presetSelect.addEventListener("change", (e) => {
    applyUiPreset(e.target.value);
    sendEqUpdate();
  });

  eqBands.forEach(b => {
    b.slider.addEventListener("input", () => {
      updateEqLabels();
      debounceSendEqUpdate();
    });
  });

  clearBassSlider.addEventListener("input", () => {
    updateEqLabels();
    debounceSendEqUpdate();
  });

  const debounceSendEqUpdate = debounce(sendEqUpdate, 75);

  function sendEqUpdate() {
    const payload = {
      preset: presetSelect.value,
      bands: eqBands.map(b => parseInt(b.slider.value, 10)),
      clearBass: parseInt(clearBassSlider.value, 10)
    };
    invoke("set_equalizer", { payload }).then(updateWireConsole).catch(console.error);
  }

  // Quick Sound Profile Chips
  const profileChips = document.querySelectorAll(".profile-chip");
  profileChips.forEach(chip => {
    chip.addEventListener("click", () => {
      profileChips.forEach(c => c.classList.remove("active"));
      chip.classList.add("active");
      const profileName = chip.dataset.profile;

      if (profileName.startsWith("Focus")) {
        setUiAncMode("NoiseCanceling");
        applyUiPreset("Excited");
      } else if (profileName.startsWith("Office")) {
        setUiAncMode("AmbientSound", 12, true);
        applyUiPreset("Vocal");
      } else if (profileName.startsWith("Cinema")) {
        setUiAncMode("NoiseCanceling");
        applyUiPreset("BassBoost");
      } else {
        setUiAncMode("Off");
        applyUiPreset("Off");
      }

      invoke("apply_sound_profile", { profileName }).then(updateWireConsole).catch(console.error);
    });
  });

  // Diagnostics & Live Connection Management
  const connectionPill = document.getElementById("connection-pill");
  const statusDot = document.getElementById("status-dot");
  const connectionText = document.getElementById("connection-text");
  const batteryPct = document.getElementById("battery-pct");
  const batteryFill = document.getElementById("battery-fill");
  const batteryStatus = document.getElementById("battery-status");
  const connectionBanner = document.getElementById("connection-banner");
  const bannerTitle = document.getElementById("banner-title");
  const bannerBody = document.getElementById("banner-body");
  const bannerSteps = document.getElementById("banner-steps");
  const diagSummary = document.getElementById("diag-summary");

  function applyDiagnostics(diag) {
    if (!diag) return;

    statusDot.className = "status-dot";
    if (diag.status === "connected") {
      statusDot.classList.add("connected");
      connectionText.textContent = "Connected";
      if (batteryStatus) {
        batteryStatus.textContent = diag.message || "Bluetooth Stereo A2DP • Active";
      }
      if (diag.battery_percent) {
        if (batteryPct) batteryPct.textContent = `${diag.battery_percent}%`;
        if (batteryFill) batteryFill.style.width = `${diag.battery_percent}%`;
      }
      if (connectionBanner) connectionBanner.style.display = "none";
      if (diagSummary) {
        diagSummary.textContent = `Connected: ${diag.device_name || "Sony Headphones"} (${diag.mac_address || "A2DP/RFCOMM"})`;
        diagSummary.style.color = "var(--green)";
      }
    } else if (diag.status === "ble_only") {
      statusDot.classList.add("ble-only");
      connectionText.textContent = "BLE Only (Audio Disconnected)";
      if (batteryPct) batteryPct.textContent = "--";
      if (batteryFill) batteryFill.style.width = "0%";
      if (batteryStatus) batteryStatus.textContent = "LE Beacon Paired • Audio Not Connected";

      if (connectionBanner) {
        connectionBanner.className = "connection-banner";
        connectionBanner.style.display = "flex";
        if (bannerTitle) bannerTitle.textContent = `Bluetooth Low Energy (${diag.ble_device_name || "LE_WH-1000XM3"}) Detected`;
        if (bannerBody) {
          bannerBody.textContent = "Windows is paired to the Bluetooth Low Energy beacon only. In this mode, no audio is played and Sony MDR headphone controls cannot connect.";
        }
        if (bannerSteps) {
          bannerSteps.innerHTML = "";
          (diag.troubleshooting_steps || [
            "Turn headphones off.",
            "Hold Power button for 7 seconds until you hear 'Bluetooth pairing'.",
            "Open Windows Settings and pair 'WH-1000XM3' (headphone icon, NOT LE_).",
            "Click 'Re-Scan Bluetooth'."
          ]).forEach(step => {
            const li = document.createElement("li");
            li.textContent = step;
            bannerSteps.appendChild(li);
          });
        }
      }
      if (diagSummary) {
        diagSummary.textContent = `Notice: ${diag.message}`;
        diagSummary.style.color = "#fbbf24";
      }
    } else {
      statusDot.classList.add("disconnected");
      connectionText.textContent = "Disconnected";
      if (batteryPct) batteryPct.textContent = "--";
      if (batteryFill) batteryFill.style.width = "0%";
      if (batteryStatus) batteryStatus.textContent = "No Sony Audio Device Active";

      if (connectionBanner) {
        connectionBanner.className = "connection-banner error";
        connectionBanner.style.display = "flex";
        if (bannerTitle) bannerTitle.textContent = "Sony Headphones Disconnected";
        if (bannerBody) {
          bannerBody.textContent = diag.message || "No active Sony Bluetooth audio connection found.";
        }
        if (bannerSteps) {
          bannerSteps.innerHTML = "";
          (diag.troubleshooting_steps || [
            "Ensure headphones are powered on and within Bluetooth range.",
            "If not paired yet, put headphones in pairing mode (hold Power for 7s).",
            "Pair in Windows Settings > Bluetooth & devices.",
            "Click 'Re-Scan Bluetooth'."
          ]).forEach(step => {
            const li = document.createElement("li");
            li.textContent = step;
            bannerSteps.appendChild(li);
          });
        }
      }
      if (diagSummary) {
        diagSummary.textContent = `Status: ${diag.message}`;
        diagSummary.style.color = "#f87171";
      }
    }
  }

  function refreshDiagnostics() {
    invoke("get_connection_diagnostics").then(applyDiagnostics).catch(console.error);
  }

  // Initial scan immediately on startup
  refreshDiagnostics();
  // Periodic poll every 5 seconds for live status changes
  setInterval(refreshDiagnostics, 5000);

  // Settings & Banner Navigation Buttons
  const openBtSettings = () => invoke("open_windows_settings", { page: "bluetooth" }).catch(console.error);
  const openSoundSettings = () => invoke("open_windows_settings", { page: "sound" }).catch(console.error);

  const btnBannerBt = document.getElementById("btn-banner-bluetooth");
  if (btnBannerBt) btnBannerBt.addEventListener("click", openBtSettings);

  const btnOpenBtSettings = document.getElementById("btn-open-bt-settings");
  if (btnOpenBtSettings) btnOpenBtSettings.addEventListener("click", openBtSettings);

  const btnBannerSound = document.getElementById("btn-banner-sound");
  if (btnBannerSound) btnBannerSound.addEventListener("click", openSoundSettings);

  const btnOpenSoundSettings = document.getElementById("btn-open-sound-settings");
  if (btnOpenSoundSettings) btnOpenSoundSettings.addEventListener("click", openSoundSettings);

  const btnRunDiag = document.getElementById("btn-run-diag");
  if (btnRunDiag) btnRunDiag.addEventListener("click", refreshDiagnostics);

  const btnBannerReconnect = document.getElementById("btn-banner-reconnect");
  if (btnBannerReconnect) {
    btnBannerReconnect.addEventListener("click", () => {
      btnBannerReconnect.textContent = "Scanning...";
      invoke("trigger_reconnect").then((res) => {
        refreshDiagnostics();
        btnBannerReconnect.textContent = "Re-Scan Bluetooth";
      }).catch(() => {
        refreshDiagnostics();
        btnBannerReconnect.textContent = "Re-Scan Bluetooth";
      });
    });
  }

  // Reconnect & Audio Enforce Buttons
  document.getElementById("btn-reconnect").addEventListener("click", () => {
    connectionText.textContent = "Scanning...";
    invoke("trigger_reconnect").then((res) => {
      updateWireConsole(res);
      refreshDiagnostics();
    }).catch(err => {
      console.error(err);
      refreshDiagnostics();
    });
  });

  document.getElementById("btn-fix-audio").addEventListener("click", () => {
    const btn = document.getElementById("btn-fix-audio");
    btn.textContent = "Routing...";
    invoke("enforce_a2dp").then((res) => {
      btn.textContent = "Enforced A2DP ✓";
      if (res && res.message) {
        document.getElementById("status-fine").textContent = res.message;
      }
      setTimeout(() => { btn.textContent = "Enforce A2DP"; }, 2000);
    }).catch(console.error);
  });

  // Settings Toggles
  document.getElementById("autostart-toggle").addEventListener("change", (e) => {
    invoke("set_autostart", { enabled: e.target.checked }).catch(console.error);
  });

  document.getElementById("touch-sensor-toggle").addEventListener("change", (e) => {
    invoke("set_touch_sensor", { enabled: e.target.checked }).then(updateWireConsole).catch(console.error);
  });

  document.getElementById("auto-off-select").addEventListener("change", (e) => {
    invoke("set_auto_off", { minutes: parseInt(e.target.value, 10) }).then(updateWireConsole).catch(console.error);
  });

  // Guided Tools & Maintenance
  const btnReopenWizard = document.getElementById("btn-reopen-wizard");
  if (btnReopenWizard) {
    btnReopenWizard.addEventListener("click", () => {
      onboardingModal.style.display = "flex";
      invoke("relaunch_setup_wizard").catch(console.error);
    });
  }

  const btnRunProbe = document.getElementById("btn-run-probe");
  if (btnRunProbe) {
    btnRunProbe.addEventListener("click", () => {
      invoke("run_diagnostic_probe_in_terminal").catch(console.error);
    });
  }

  const btnUninstallApp = document.getElementById("btn-uninstall-app");
  if (btnUninstallApp) {
    btnUninstallApp.addEventListener("click", () => {
      if (confirm("Are you sure you want to uninstall SonyBridge? This will launch the uninstallation wizard.")) {
        invoke("trigger_uninstall").catch(console.error);
      }
    });
  }

  // Guided Headphone Pairing Assistant
  const pairingModal = document.getElementById("modal-pairing-assistant");
  const btnOpenPairing = document.getElementById("btn-open-pairing");
  const btnBannerPair = document.getElementById("btn-banner-pair");
  const btnTabPair = document.getElementById("btn-tab-pair");
  const btnClosePairing = document.getElementById("btn-close-pairing");
  const btnFinishPairing = document.getElementById("btn-finish-pairing");
  const btnLaunchDeviceWizard = document.getElementById("btn-launch-device-wizard");
  const btnLaunchAddDevice = document.getElementById("btn-launch-add-device");
  const btnPairingOpenSettings = document.getElementById("btn-pairing-open-settings");
  const pairingLiveDot = document.getElementById("pairing-live-dot");
  const pairingLiveDesc = document.getElementById("pairing-live-desc");
  const pairingStatusText = document.getElementById("pairing-status-text");

  let pairingInterval = null;

  function showPairingModal() {
    if (pairingModal) {
      pairingModal.style.display = "flex";
      startPairingLiveWatch();
    }
  }

  function hidePairingModal() {
    if (pairingModal) {
      pairingModal.style.display = "none";
      if (pairingInterval) {
        clearInterval(pairingInterval);
        pairingInterval = null;
      }
    }
  }

  function startPairingLiveWatch() {
    if (pairingInterval) clearInterval(pairingInterval);
    const check = () => {
      invoke("get_connection_diagnostics").then((diag) => {
        applyDiagnostics(diag);
        if (diag && diag.status === "connected") {
          if (pairingLiveDot) {
            pairingLiveDot.className = "status-dot connected";
          }
          if (pairingLiveDesc) {
            pairingLiveDesc.textContent = `Success! Connected to ${diag.device_name || "WH-1000XM3"}`;
            pairingLiveDesc.style.color = "var(--green)";
          }
          if (pairingStatusText) {
            pairingStatusText.textContent = "Your headphones are paired, connected, and active in Windows!";
          }
        } else if (diag && diag.status === "ble_only") {
          if (pairingLiveDot) {
            pairingLiveDot.className = "status-dot ble-only";
          }
          if (pairingLiveDesc) {
            pairingLiveDesc.textContent = "Found 'LE_WH-1000XM3'. Waiting for 'WH-1000XM3' Audio...";
            pairingLiveDesc.style.color = "#fbbf24";
          }
        } else {
          if (pairingLiveDot) {
            pairingLiveDot.className = "status-dot standby";
          }
          if (pairingLiveDesc) {
            pairingLiveDesc.textContent = "Scanning for discoverable WH-1000XM3...";
            pairingLiveDesc.style.color = "var(--accent-gold)";
          }
        }
      }).catch(console.error);
    };
    check();
    pairingInterval = setInterval(check, 2000);
  }

  if (btnOpenPairing) btnOpenPairing.addEventListener("click", showPairingModal);
  if (btnBannerPair) btnBannerPair.addEventListener("click", showPairingModal);
  if (btnTabPair) btnTabPair.addEventListener("click", showPairingModal);
  if (btnClosePairing) btnClosePairing.addEventListener("click", hidePairingModal);
  if (btnFinishPairing) btnFinishPairing.addEventListener("click", hidePairingModal);

  if (btnLaunchDeviceWizard) {
    btnLaunchDeviceWizard.addEventListener("click", () => {
      invoke("launch_pairing_wizard").catch(console.error);
    });
  }

  if (btnLaunchAddDevice) {
    btnLaunchAddDevice.addEventListener("click", () => {
      invoke("open_add_device_flyout").catch(console.error);
    });
  }

  if (btnPairingOpenSettings) {
    btnPairingOpenSettings.addEventListener("click", openBtSettings);
  }

  updateEqLabels();
});
