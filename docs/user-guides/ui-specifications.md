# UI/UX Specifications Document
## Handy Launcher

**Version:** 1.0  
**Date:** March 16, 2026  
**Status:** Draft

---

## 1. Design Principles

### 1.1 Philosophy
- **Invisible by default:** Handy Launcher runs in the background; users only see it when they need it
- **Zero friction:** Auto-advance through steps, smart defaults, minimal clicks
- **Human-readable:** Technical details hidden; outcome-based language
- **Native feel:** Platform-appropriate styling, system tray integration

### 1.2 User Mental Model
Users think: *"I want local transcription in Handy"* — not *"I want to install Ollama"*

**Our UI mirrors this:**
- Setup asks: "Enable local transcription?" (not "Install Ollama?")
- Model selection asks: "How accurate do you want it?" (not "Which model?")
- Status shows: "Local transcription ready" (not "Ollama running on port 63452")

---

## 2. Application Flow

### 2.1 Entry Points

| Scenario | Behavior |
|----------|----------|
| **First launch** | Full-screen setup wizard |
| **Subsequent launch (click icon)** | Opens status dashboard (or brings existing window to front) |
| **System tray click** | Context menu or toggle dashboard visibility |
| **Auto-start on login** | Tray icon only, no window |

### 2.2 State Machine

```
┌─────────────┐     First run      ┌───────────────┐
│  Not Setup  │ ──────────────────>│ Setup Wizard  │
└─────────────┘                    └───────┬───────┘
     ▲                                   │
     │                                   │ Complete
     │                                   ▼
     │                          ┌───────────────┐
     │                          │   Dashboard   │
     │                          │  (Tray icon)  │
     │                          └───────┬───────┘
     │                                  │
     │         Click Reconfigure        │
     └──────────────────────────────────┘
```

---

## 3. Setup Wizard

### 3.1 Wizard Structure

**Total steps:** 3 (with intelligent skipping)

| Step | Name | User Input Required? | Skip Condition |
|------|------|---------------------|----------------|
| 1 | Welcome | Yes ("Get Started" button) | Never |
| 2 | System Check | No | Auto-advance on pass |
| 3 | Quality Selection | Yes (model profile) | Never (but pre-select recommended) |

**Post-wizard:** Download/install happens automatically, then auto-close to tray.

### 3.2 Step 1: Welcome

```
┌─────────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────────┐     │
│  │                                                     │     │
│  │              [Logo: Sound wave + Shield]            │     │
│  │                                                     │     │
│  │         Enable Local Transcription                  │     │
│  │                                                     │     │
│  │   Your words stay on your device. No cloud.       │     │
│  │   No subscription. Just better transcription.     │     │
│  │                                                     │     │
│  │         [         Get Started         ]            │     │
│  │                                                     │     │
│  │   ───  ───  ───                                   │     │
│  │                                                     │     │
│  │   [Advanced]  [Learn more about privacy]           │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                    [T][X]    │
└─────────────────────────────────────────────────────────────┘
```

**Elements:**
- Centered card layout with ample whitespace
- Primary CTA: "Get Started" (single button, no alternatives to reduce decision fatigue)
- Secondary: "Advanced" toggle (reveals technical details for power users)
- Tertiary: "Learn more" link (opens external docs)

**Behavior:**
- Click "Get Started" → Run system check → Advance to Step 2
- Click "Advanced" → Toggle showing: custom install path, manual Ollama selection

### 3.3 Step 2: System Check

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│         Checking your device...                             │
│                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│   │  ✅  RAM    │  │  ✅  Disk   │  │  ✅  OS     │         │
│   │             │  │             │  │             │         │
│   │   16 GB    │  │   45 GB    │  │ Windows 11  │         │
│   │  detected   │  │  available │  │  supported  │         │
│   └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                             │
│   All good!                                                  │
│                                                             │
│                   [   Continue   ]                           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**States:**

| State | Visual | Action |
|-------|--------|--------|
| Checking | Spinner + gray text | None (auto-run) |
| Pass | Green checkmark, auto-advance after 500ms delay | Continue button shown briefly, then auto-advance |
| Warning (e.g., low RAM) | Yellow icon, inline message | Show "Continue anyway" button |
| Fail | Red icon, helpful message | Disable Continue, show troubleshooting |

**Auto-advance:** If all checks pass, wizard advances to Step 3 after 500ms (long enough to see success, short enough to not feel slow).

### 3.4 Step 3: Quality Selection

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│         Choose transcription quality                        │
│                                                             │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│   │    ⚡ FAST      │  │  ✓ RECOMMENDED  │  │   🎯 ACCURATE   │ │
│   │                 │  │                 │  │                 │ │
│   │  Quick results  │  │  Balanced      │  │  Best quality   │ │
│   │  Good for notes │  │  All-around    │  │  Meeting mins   │ │
│   │                 │                 │  │                 │ │
│   │  • 1.3 GB       │  │  • 3.8 GB      │  │  • 7 GB       │ │
│   │  • 2 GB RAM     │  │  • 6 GB RAM    │  │  • 8 GB RAM ⚠️ │ │
│   │  • Download: ~2m│  │  • Download: ~6m│ │  • Download: ~12m│ │
│   │                 │  │                 │  │                 │ │
│   │  [   SELECT   ] │  │  [   SELECT   ] │  │                 │ │
│   └─────────────────┘  └─────────────────┘  │  UNSUITABLE    │ │
│                                             │  8 GB RAM req'd│ │
│                                             └─────────────────┘ │
│                                                             │
│   ℹ️  Using: llama3.2:1b  [Show details]                    │
│   Estimated setup time: 2 minutes                           │
│                                                             │
│                        [   Confirm & Download   ]           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Card Design:**

| Element | Specification |
|---------|--------------|
| Layout | 3-column grid, responsive (stack on narrow windows) |
| Card padding | 24px internal, 16px between cards |
| Selection state | 2px primary border, subtle background tint |
| Recommended badge | Small pill label, "RECOMMENDED" text |
| Unsuitable card | Grayed (opacity 0.6), no Select button, shows requirement |

**Technical Details Toggle:**
- Hidden by default (`.technical-details { display: none; }`)
- Clicking "Show details" reveals: model name, parameters, quantization level

**Confirm & Download:**
- Primary button, disabled until selection made (with recommended pre-selected)
- Spawns download progress indicator (progress shown in dashboard/tray, wizard auto-closes)

---

## 4. Download Progress

### 4.1 Background Download Behavior

Wizard closes immediately after "Confirm & Download" click. Progress shown via:

**Option A: System Tray (primary)**
- Windows: Tray icon shows download percentage in tooltip
- macOS: Menubar icon shows percentage directly

**Option B: Dashboard Window (if user re-opens)**
- Same progress display as tray, but in window form

### 4.2 System Tray States

| State | Icon | Tooltip |
|-------|------|---------|
| Idle / Ready | Checkmark | "Handy Launcher: Local transcription ready" |
| Downloading | Animated arrow | "Downloading model: 45%" |
| Installing | Spinner | "Installing..." |
| Error | Warning | "Setup error — click for details" |
| Ollama stopped | Pause | "Ollama stopped — click to start" |

### 4.3 Dashboard Download View (if opened during download)

```
┌─────────────────────────────────────────────────────────────┐
│  Handy Launcher                                      [=][X] │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐     │
│  │                                                     │     │
│  │              Downloading model...                   │     │
│  │                                                     │     │
│  │         [████████████████░░░░░░░░]  67%           │     │
│  │                                                     │     │
│  │         2.5 GB / 3.8 GB downloaded                │     │
│  │         About 2 minutes remaining                 │     │
│  │                                                     │     │
│  │         Speed: 5.2 MB/s                           │     │
│  │                                                     │     │
│  │         [   Cancel Download   ]                   │     │
│  │                                                     │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Progress bar:**
- Determinate, smooth animation
- Color: system accent color
- Shows percentage numerically

**Cancel behavior:**
- Confirm dialog: "Cancel download?" / "Resume later?"
- Cancel → Delete partial download, return to Quality Selection

---

## 5. Status Dashboard

### 5.1 Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Handy Launcher                                      [=][X] │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐     │
│  │  Local transcription                                │     │
│  │                                                     │     │
│  │  ● Running and ready                    [Test]      │     │
│  │                                                     │     │
│  │  Model: llama3.2:1b (Fast profile)                  │     │
│  │  Last used: Today, 2:34 PM                          │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                             │
│  Quick Actions:                                            │
│                                                             │
│  [  🔄 Switch Model  ]  [  🔍 Test Connection  ]            │
│                                                             │
│  [  📋 View Logs     ]  [  ⚙️ Reconfigure     ]            │
│                                                             │
│  ─────────────────────────────────────────────────────      │
│                                                             │
│  Troubleshooting                                           │
│                                                             │
│  Ollama is running on port 63452                          │
│  [  Open Troubleshooting Mode  ]                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Card sections:**

| Section | Content |
|---------|---------|
| **Status** | Large indicator (running/stopped), model name, last active time |
| **Quick Actions** | 4 buttons in 2x2 grid |
| **Troubleshooting** | Collapsed by default, expands to show technical details |

### 5.2 Status Indicators

| State | Icon | Text | Color |
|-------|------|------|-------|
| Ready | Solid circle | "Running and ready" | Green |
| Stopped | Pause icon | "Stopped" | Gray |
| Error | Warning | "Error — check logs" | Red |
| Loading | Spinner | "Starting..." | Blue |

### 5.3 Quick Actions

| Button | Action |
|--------|--------|
| **Switch Model** | Opens model selection (Step 3), downloads new model if needed |
| **Test Connection** | Makes test API call, shows latency toast: "Connected (45ms)" |
| **View Logs** | Opens log viewer window |
| **Reconfigure** | Restarts setup wizard |

---

## 6. Troubleshooting Mode

### 6.1 Access

**Toggle:** "[ ] Show technical details" at bottom of dashboard

**Power user shortcut:** Hold `Alt` key to reveal "Troubleshooting" button instantly

### 6.2 Troubleshooting View

```
┌─────────────────────────────────────────────────────────────┐
│  Troubleshooting Mode                              [Close]  │
│                                                             │
│  Ollama Details                                            │
│  ┌─────────────────────────────────────────────────────┐     │
│  │  Version: 0.6.7                                     │     │
│  │  PID: 45234                                         │     │
│  │  Port: 63452                                        │     │
│  │  Base URL: http://127.0.0.1:63452                   │     │
│  │  Models: 2 (llama3.2:1b, phi4:mini)                 │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                             │
│  Logs (last 50 lines)                                       │
│  ┌─────────────────────────────────────────────────────┐     │
│  │ [10:34:22] INFO: Ollama started                     │     │
│  │ [10:34:22] INFO: Listening on 127.0.0.1:63452      │     │
│  │ ...                                                 │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                             │
│  [Refresh]  [Export Logs]  [Restart Ollama]  [Kill Process]  │
│                                                             │
│  Advanced:                                                 │
│  [View RAW settings_merge.json]  [Force port change]          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Actions:**

| Action | Behavior |
|--------|----------|
| **Refresh** | Reload all status indicators |
| **Export Logs** | Create ZIP with launcher.log + Ollama logs |
| **Restart Ollama** | Graceful stop → start |
| **Kill Process** | Force kill with confirmation |
| **Force port change** | Manually specify port, restart Ollama |

---

## 7. Error Handling

### 7.1 Error Patterns

| Severity | Pattern | Example |
|----------|---------|---------|
| **Warning** | Inline banner, can proceed | "Low disk space — consider Fast profile" |
| **Error** | Modal dialog, requires action | "Download failed. Retry or switch model?" |
| **Critical** | Blocking modal, must dismiss | "Setup incomplete. Local transcription unavailable." |

### 7.2 Error Messages

**Principles:**
- Say what happened
- Say why it matters (to the user's goal)
- Provide clear next action

**Examples:**

| Technical Reality | User-Facing Message |
|-------------------|---------------------|
| "Connection refused on port 63452" | "Ollama isn't responding. Restarting..." |
| "Model llama3.2:1b not found" | "Downloading the Fast model..." |
| "HTTP 503: model still loading" | "Warming up — almost ready" |
| "JSON parse error in settings_store.json" | "Couldn't read Handy settings. Please check Handy is closed." |

---

## 8. Visual Design Specifications

### 8.1 Platform-Specific Native Styling

**Windows:**
- Font: `Segoe UI Variable Text` (Win 11) or `Segoe UI` (Win 10)
- Window chrome: Standard Windows frame with dark mode support
- Context menus: Native Windows style
- System tray: Standard notification area icon

**macOS:**
- Font: `-apple-system` (SF Pro Text)
- Window chrome: Standard macOS with vibrancy (if supported)
- Menubar: Percentage shown in icon or dropdown menu, not window
- Context menus: Native NSMenu

### 8.2 Color Scheme

**Light Mode:**
- Background: `#FFFFFF`
- Card background: `#F7F7F7`
- Text primary: `#1A1A1A`
- Text secondary: `#6E6E6E`
- Accent: System accent color (blue by default)
- Success: `#34C759` (Apple green / Windows success)
- Warning: `#FF9500`
- Error: `#FF3B30`

**Dark Mode:**
- Background: `#1C1C1E`
- Card background: `#2C2C2E`
- Text primary: `#FFFFFF`
- Text secondary: `#8E8E93`
- Accent: System accent color

### 8.3 Typography Scale

| Level | Size | Weight | Use |
|-------|------|--------|-----|
| Title | 24px | Semibold | Page titles |
| Heading | 18px | Semibold | Card headers |
| Body | 14px | Regular | Primary text |
| Caption | 12px | Regular | Secondary details |
| Button | 14px | Medium | Labels |

### 8.4 Spacing & Sizing

- Window min size: 600px × 400px
- Card padding: 24px
- Button height: 36px
- Button padding: 16px horizontal
- Border radius: 8px (cards), 6px (buttons)

---

## 9. System Tray / Menubar Integration

### 9.1 Windows (System Tray)

**Icon sizes:** 16×16, 20×20, 24×24, 32×32 (for high DPI)

**Context Menu:**
```
┌──────────────────────────────┐
│ Local transcription: Ready  │
│ ---------------------------  │
│ Test Connection             │
│ Open Dashboard              │
│ ---------------------------  │
│ Start Ollama              │
│ Stop Ollama               │
│ ---------------------------  │
│ Exit                       │
└──────────────────────────────┘
```

### 9.2 macOS (Menubar)

**Icon:** Template image (white/black for dark/light mode)

**Dropdown Menu:**
```
┌──────────────────────────────┐
│ ● Local transcription ready  │
│    Model: llama3.2:1b        │
│ ---------------------------  │
│ Test Connection (⌘T)        │
│ Open Dashboard (⌘O)         │
│ ---------------------------  │
│ Start/Stop Ollama          │
│ ---------------------------  │
│ Preferences... (⌘,)         │
│ Quit (⌘Q)                   │
└──────────────────────────────┘
```

---

## 10. Responsive Considerations

### 10.1 Window Sizes

| Size | Adaptation |
|------|-----------|
| ≥800px wide | 3-column model cards |
| 600-799px | 2-column cards + stacked |
| <600px | Single column, scrollable |

### 10.2 Minimum Viable Window

All functionality must work at 600×400 (the minimum Tauri window size). Cards stack vertically, model selection uses radio-style list instead of grid.

---

## 11. Keyboard Shortcuts

### 11.1 Global Shortcuts (when app is focused)

| Shortcut | Action |
|----------|--------|
| `Esc` | Close window (to tray), cancel dialogs |
| `Cmd/Ctrl + ,` | Open Preferences |
| `Cmd/Ctrl + Q` | Quit application |
| `Alt` | Reveal troubleshooting options |

### 11.2 Wizard Navigation

| Shortcut | Action |
|----------|--------|
| `Enter` | Activate primary button |
| `Tab` | Move focus between elements |
| `Space` | Toggle checkbox, activate focused button |

---

## 12. Accessibility

### 12.1 Requirements

- Full keyboard navigation (Tab order, focus states)
- Screen reader labels for all interactive elements
- Minimum contrast ratio: 4.5:1 for text
- Focus indicators: 2px outline with 2px offset
- Reduced motion: Respect `prefers-reduced-motion`

### 12.2 Accessibility Labels

| Element | Screen Reader Label |
|---------|---------------------|
| Model card | "Fast profile: Quick results, 1.3 gigabytes, 2 gigabytes RAM required" |
| Status indicator | "Local transcription is running and ready" |
| Progress bar | "Download progress: 67 percent complete" |
| Tray icon | "Handy Launcher: Local transcription ready" |

---

## 13. Implementation Notes

### 13.1 Svelte Components Structure

```
frontend/src/
├── routes/
│   ├── SetupWizard.svelte      # Wizard container
│   ├── StatusDashboard.svelte  # Main dashboard
│   └── Troubleshooting.svelte  # Advanced view
├── components/
│   ├── StepWelcome.svelte      # Wizard step 1
│   ├── StepSystemCheck.svelte  # Wizard step 2
│   ├── StepQualitySelect.svelte # Wizard step 3
│   ├── ModelCard.svelte        # Quality selection card
│   ├── ProgressBar.svelte      # Download progress
│   ├── StatusIndicator.svelte  # Running/stopped indicator
│   ├── LogViewer.svelte        # Scrollable log display
│   └── TrayMenu.svelte         # System tray/menubar menu
└── stores/
    ├── setup.ts                # Wizard state
    ├── ollama.ts               # Ollama status
    └── tray.ts                 # Tray visibility
```

### 13.2 State Management

**Wizard flow controlled by:**
```typescript
// stores/setup.ts
interface SetupState {
  step: 'welcome' | 'system-check' | 'quality-select' | 'complete';
  systemInfo: SystemInfo | null;
  selectedProfile: ModelProfile | null;
  downloadProgress: DownloadProgress | null;
  error: SetupError | null;
}
```

**Auto-advance logic:**
```typescript
// In StepSystemCheck.svelte
$: if (systemCheck.status === 'passed') {
  setTimeout(() => gotoStep('quality-select'), 500);
}
```

---

## 14. References

- [Tauri Window Management](https://tauri.app/v1/guides/features/window-customization/)
- [Tauri System Tray](https://tauri.app/v1/guides/features/system-tray/)
- [Svelte Transition API](https://svelte.dev/docs#template-syntax-svelte-transition) (for wizard step animations)
- [Apple Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/)
- [Microsoft Fluent Design System](https://www.microsoft.com/design/fluent/)

---

*Document created: March 16, 2026*
*Based on requirements v1.0 and architecture v1.1*
