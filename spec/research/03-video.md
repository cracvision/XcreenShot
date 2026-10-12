# Snagit video recording, video editing and GIF: feature inventory from public sources

**How this was built.** I used only TechSmith tutorials (techsmith.com/learn), support.techsmith.com articles, the official yearly version histories (Windows and Mac, 2019–2026), the official "What's new" changelog (all 13 pages), product feature pages, and a few third-party guides and reviews. I did not download, install or inspect any Snagit binary, installer or file. Every description below is paraphrased.

**Limits of the research:**
- The 200-search web search budget for this run ran out near the end. A few details are still unconfirmed:
  - the sub-options inside the Highlight Cursor and Click Animation panels (size, opacity, shape);
  - the exact current names of the Restart and Cancel buttons on the recording toolbar.
- Downloading public help PDFs with curl was blocked by the proxy (403), so I read the PDFs through WebFetch only.

**Notation:**
- Win = Windows, Mac = macOS, Both = both, (?) = uncertain or inferred.
- Version numbers are Snagit's year-based numbers. 2026.x releases are branded "Snagit" with no year; Windows 2026.0.0 notes say Snagit joined the "Camtasia" portfolio.
- Source codes like [S1] or [W24] are defined in the reference table at the end, which is also the deduplicated list of every URL consulted.

---

## A. Starting a recording and capture setup

**Video tab (Capture Window)**
- A dedicated tab holds all video settings: Open In, Selection, Share, Camera, Screen Draw, Cursor, Microphone, System Audio and Presets.
- The Capture button, or the global capture hotkey, starts area selection.
- Platform: Both. Version: long-standing.
- Sources: [S1][S19][S18]

**All-in-One capture → Video Capture button**
- From the All-in-One tab, the user selects an area, then picks Video Capture from a small toolbar to record instead of taking an image.
- A preference can make recording start immediately after Video Capture is clicked.
- Win: All-in-One remembers mic, camera, cursor highlight and click-animation settings from the last recording (fixes 2023.0.0 and 2024.0.1).
- Mac: All-in-One remembers webcam blur (2024.2.0).
- Platform: Both.
- Sources: [S18][S11][W23][W24][M24]

**Capture Widget**
- An alternative launcher that docks as a tab on the screen edge with limited options. It can start video recordings.
- Platform: Both.
- Sources: [S10][S11][W22]

**Global capture hotkey**
- Starts a capture using the current Capture Window settings, which can be video.
- Defaults: Print Screen (Win), Control+Shift+C (Mac). The hotkey is user-changeable.
- On Windows 11 the Print Screen key may be claimed by the Snipping Tool; the OS setting must be turned off.
- Platform: Both.
- Sources: [S17][S12]

**Mac dedicated Video Capture hotkey**
- Control+Shift+V starts a video capture directly. The same keys stop a recording.
- Platform: Mac.
- Sources: [S12][S34]

**Capture presets for video**
- Save video settings (camera, mic, share destination, etc.) as named presets, each with its own hotkey.
- Presets can be reordered, edited, imported or exported (`.snagpresets` on Win), and multi-selected.
- A built-in preset for "open in Camtasia Editor" was added in Win 2026.2.0 and Mac 2026.3.0.
- Fixes: Win 2026.4.0 made presets respect the selected microphone; Mac presets remember assigned shortcuts (2024.0.1).
- Mac 2025.4.0: drag-to-reorder presets.
- Platform: Both.
- Sources: [S16][S1][W26][M26][M25][S29]

**Repeat Last Capture**
- Re-runs the previous capture type.
- Mac default Control+Shift+R. Win has no default (assignable).
- Platform: Both.
- Sources: [S12]

**Ready to Record window**
- After the area is selected, a panel summarizes current devices and settings before recording.
- Since Win 2026.1.0 it also shows whether the video will open in Snagit Editor or Camtasia Editor.
- Can be disabled in preferences.
- Platform: Both. Version: Win 2024.1.2; Mac 2024.2.3/2024.2.4.
- Sources: [W24][M24][S10][S11][W26][S29]

**Automatically start video recording (preference)**
- Skips the Record click: recording starts as soon as the area or Video Capture is chosen. Off by default.
- Platform: Both.
- Sources: [S10][S11]

**Show video countdown (preference)**
- A 3-second countdown before recording begins.
- Mac fixes cover pausing during the countdown and the countdown leaking into the video.
- Platform: Both.
- Sources: [S10][S11][M22][M23]

**Switch an accidental video capture to an image capture**
- Converts a video capture that was started by mistake into an image capture.
- Platform: Mac. Version: 2021.3.0.
- Sources: [M21]

**Learning Center tour "Record a Video"**
- Guided in-product tour. It is disabled when video recording is disabled by deployment.
- Platform: Win. Version: 2024.0.3.
- Sources: [W24]

---

## B. Recording area selection and dimensions

**Region selection**
- Drag orange crosshairs to select an area.
- Adjust the area by dragging handles, moving it, or typing Width/Height in pixels.
- A magnifier helps with pixel accuracy.
- Win: arrow keys nudge the crosshair 1 px; Enter captures; Shift+Enter starts a region selection.
- Platform: Both.
- Sources: [S1][S12][S28]

**Window selection**
- Win: crosshairs snap to window and region boundaries.
- Mac (2026.0.0): records one chosen app window and keeps following it when it is moved, resized or covered, without capturing other windows. This was an experiment in 2025.0.0; Mac 2026.0.1 fixed the toolbar following the window.
- Platform: Both, with different behavior. Version: Mac 2025.0.0 (experimental), 2026.0.0 (GA).
- Sources: [S1][M25][M26]

**Fullscreen**
- A Fullscreen button records the whole display. On Mac, one monitor only.
- Mac also offers Fullscreen in the Selection menu.
- Platform: Both.
- Sources: [S1][S2][R5]

**Selection gear (more options)**
- A gear icon next to Selection opens more options.
- On Mac it includes Selection Properties for a fixed region size and position (?).
- Platform: Both (?).
- Sources: [S1][R5]

**Aspect-ratio lock while dragging**
- Sources disagree:
  - Current Win hotkeys guide: hold Ctrl for 16:9, Ctrl+Shift for 4:3, Shift for 1:1 square.
  - Current video tutorial: Ctrl (Win) or Shift (Mac) for 16:9, Shift for 4:3 (Win).
  - Legacy page: Ctrl for 4:3, Shift for 16:9.
  - Mac Help PDF: Shift draws a square.
- Platform: Both.
- Sources: [S12][S1][S28][R5]

**Video selection guides (G key)**
- Toggles guides for common sizes: 1920×1080, 1280×720, 640×480 and 320×240.
- The selection border highlights when it matches a guide.
- Platform: Both.
- Sources: [S1][S12][R4]

**Width/Height fields on the recording toolbar**
- Edit the recording size numerically before recording.
- Fixes: Mac 2020.0.1 and 2021.4.7; Win 2024.1.4 (Esc crash).
- Platform: Both.
- Sources: [M20][M21][W24]

**Move the recording area and toolbar before recording**
- The toolbar can be repositioned before recording (Mac 2022.1.0).
- Win 2023.1.0: the toolbar defaults to just below or above the area.
- Platform: Both.
- Sources: [M22][W23]

**Scale captures down to 1920×1080 (Win) / Downsample Retina video recordings (Mac)**
- On by default; reduces output dimensions.
- Turning it off keeps text sharper but gives bigger files.
- Mac 2026.4.0 fixed the downsample setting being ignored for single-window captures.
- Platform: Win / Mac respectively.
- Sources: [S10][S11][M26]

**Dimension limits in hardware mode (?)**
- A Win troubleshooting article ties a "cannot open file" error to dimensions above 3840×2160 or below 64×64 and suggests software mode. This implies hardware encoder limits.
- Platform: Win.
- Sources: [K4]

**HDR**
- HDR color support applies to images only, not video or scrolling captures.
- Platform: Win. Version: 2024.2.2.
- Sources: [W24][S29]

---

## C. Recording toolbar and recording lifecycle

**Video Recording toolbar**
- Appears with the selection before and during recording. Toggles can be changed before or while recording.
- Controls:
  - Cursor toggle, plus a Cursor Effects dropdown
  - System Audio toggle
  - Camera toggle, plus Camera Options dropdown
  - Microphone toggle, plus Microphone Options dropdown
  - Screen Draw button
  - Timer (Mac)
  - Record, Pause/Resume, Stop
- Since 2024.0.0, each toggle is separated from its dropdown.
- Audio level meters show on the mic and system-audio buttons.
- Platform: Both.
- Sources: [S1][S2][W24][M24][W26][M23]

**Record / Pause / Resume / Stop**
- Defaults:
  - Win: Shift+F9 to record and to pause/resume; Shift+F10 to stop.
  - Mac: Control+Shift+Space to record and to pause/resume; Control+Shift+V to stop.
- Some pages list Command+Shift+Space for Mac Record.
- Platform: Both.
- Sources: [S1][S12][S34][S2]

**Restart / cancel a recording (?)**
- Several sources point to restart and cancel behavior:
  - The tutorial tips mention restarting via hotkeys in fullscreen.
  - Version notes reference restarting a recording: webcam stays on (Mac 2021.2.0), PiP position remembered on restart (Mac 2021.4.2), mic selection persists after "cancel and restart" (Win 2025.3.1), and a cancelled-recording taskbar bug (Win 2026.2.0).
  - Esc cancels a capture.
- No current page names the exact buttons.
- Platform: Both (?).
- Sources: [S1][M21][W25][W26][S12]

**Timer (auto-stop)**
- Set hours, minutes and seconds; recording stops automatically when the time is up.
- Platform: Mac. Version: 2026.4.0.
- Sources: [S1][M26]

**Elapsed recording time display**
- The toolbar shows the running recording time.
- Mac 2023.1.2 fixed it showing "--:--:--".
- Platform: Mac (Win ?).
- Sources: [M23]

**Mini or minimized toolbar for fullscreen on one monitor**
- Preference: "Minimize controls when recording fullscreen with one monitor." Shows small draggable Screen Draw, Pause and Webcam controls.
- Win 2023.0.2 added tooltips for the mini toolbar and made the mini Screen Draw toolbar movable.
- Tip from the tutorials: with two displays, drag the toolbar onto the display that isn't being recorded.
- Platform: Both.
- Sources: [S10][S11][W23][S1]

**Hide Snagit during capture / remove Snagit controls from completed captures**
- Keeps toolbars, menus, previews and tooltips out of the final video.
- Win requires Windows 10 20H1+ (2024.1.0). Mac has had it since Ventura support (2023.1.0).
- Many later fixes stop toolbar menus leaking into videos (Win/Mac 2025.1.0, Mac 2025.2.1, Mac 2026.3.2).
- Platform: Both.
- Sources: [S10][S11][W24][M23][W25][M25][M26]

**Hide desktop icons during video capture**
- Only open windows appear in the recording.
- Set on the Capture tab, or toggle from the menu-bar icon's right-click menu while recording.
- Platform: Mac. Version: 2024.3.0.
- Sources: [S11][M24][S29]

**Muted Mic Alert**
- Warns on the toolbar when speech is detected but the mic is off or muted.
- Preference: "Show alert when microphone is muted." Esc dismisses it (Mac 2025.1.0).
- Mac 2024.3.0 tuned its sensitivity for when noise removal is on.
- Platform: Both. Version: Win 2024.1.0; Mac 2024.2.0.
- Sources: [W24][M24][S10][S11][M25]

**Recording status indicator**
- Win: the tray icon changes while recording (2020.1.0).
- Mac: the menu-bar icon turns red while recording, with an option to keep it black/white.
- Platform: Both.
- Sources: [W20][S11]

**Sleep prevention**
- The computer is kept awake during recording and playback.
- Platform: Mac. Version: 2023.1.2.
- Sources: [M23]

**Zoom meeting border removal**
- The green screen-sharing border no longer appears when recording Zoom.
- Platform: Mac. Version: 2026.3.0.
- Sources: [M26]

**System screenshots taken during a recording**
- macOS system captures made during a recording are imported the next time Editor opens.
- Platform: Mac. Version: 2025.1.0.
- Sources: [M25]

---

## D. Recording quality, frame rate, encoding and engine

**Video Quality (Win)**
- Five levels in Capture Preferences. Default is Medium.
- Approximate maximum frame rates:

| Level | Max fps |
|---|---|
| Very Low | 5 |
| Low | 10 |
| Medium | 15 |
| High | 30 (4K: 20 before 2023, 30 from 2023.0.0) |
| Very High | 60 (2024.2.0+) |

- Platform: Win. Version: levels from 2021; 60 fps from 2024.2.0.
- Sources: [K3][K4][S10][W24]

**Frame Rate (Mac)**
- Settings: Low / Medium / High / Very High. Very High is 60 fps (2024.1.0).
- An adjustable maximum frame rate was added in 2020.2.1.
- Platform: Mac.
- Sources: [S11][M24][M20][M21][S29]

**Variable frame rate**
- New frames are recorded only when the screen changes, so actual fps varies up to the cap.
- True 4K60 needs a discrete GPU.
- Platform: Win (documented).
- Sources: [K3]

**Encoding: H.264 (default) or H.265**
- H.265 gives smaller files but needs more CPU and has narrower playback support. Notes say these files may not open on Windows 10.
- Platform: Mac. Version: 2025.2.0.
- Sources: [S11][M25]

**Disable hardware accelerated recording**
- Switches from hardware (GPU) encoding to software encoding to work around problems.
- Since 2023.0.0, Snagit falls back to software automatically on Intel hardware-encoder errors at 1080p or lower.
- Platform: Win. Version: 2022.1.4.
- Sources: [S10][K4][W22][W23]

**Recording engines**
- New stable engine: 2022.0.0, both platforms.
- New Win engine with 60 fps: 2024.2.0.
- Engine upgrade: Win 2025.4.0.
- Win file sizes about 10–20% smaller: 2024.1.4.
- Platform: Both.
- Sources: [W22][M22][W24][W25]

**Screen-capture method fallback (registry)**
- Win registry flag `AllowDxgiScreenCapture`; setting it to 0 changes the capture method when videos come out blank or stuck on "Initializing."
- Win audio uses Media Foundation; `AllowMfAudioCapture=0` reverts to the older WAV API.
- Platform: Win.
- Sources: [K19][K20]

**Crash or interruption recovery**
- Unsaved recordings can be recovered after a crash: Mac 2022.0.0, Win 2022.1.0.
- Mac keeps a Temporary recording folder plus an Autosaved Captures folder, and prompts "Recover" on relaunch.
- Mac 2026.1.0 added recovery for recordings set to open in Camtasia Editor.
- Platform: Both.
- Sources: [M22][W22][K10][M26]

**Maximum length**
- There is no hard limit. TechSmith recommends keeping recordings to 1 hour or less; longer ones may fail.
- Advice: record shorter clips and combine them.
- Platform: Both.
- Sources: [K1][K2][S1]

---

## E. Webcam / camera

**Camera toggle and device selection**
- Turn the camera on or off before or during recording, and pick the built-in or external device.
- The camera is on by default for new video captures: Win 2022.1.0 (admins can disable), Mac 2022.0.1.
- Platform: Both.
- Sources: [S1][S3][W22][M22]

**Picture-in-Picture (PiP)**
- The webcam overlays the screen recording.
- Placement: Top-left, Top-right, Bottom-left or Bottom-right.
- Resize by dragging a corner; drag to move between corners. On Mac it snaps to corners (2026.4.0 fix).
- The PiP location is remembered between captures.
- PiP is not a free-floating window.
- Platform: Both. Version: Mac 2021.2.0; Win 2021.3.0.
- Sources: [S3][W21][M21][M22][M26][R2]

**PiP aspect ratio 4:3 or 16:9**
- Option for the rectangular webcam view.
- Platform: Win. Version: 2021.4.0.
- Sources: [W21]

**PiP shapes**
- Circle, Rectangle, Rounded Rectangle, Square, Rounded Square.
- Platform: Both. Version: Mac 2022.0.2; Win 2022.1.1.
- Sources: [S3][S1][W22][M22]

**Fullscreen camera (webcam only)**
- Placement = Fullscreen (Win) or Fullscreen Camera (Mac). Shows only the camera; the screen is not recorded.
- Platform: Both.
- Sources: [S3][S1][K8][R4]

**Swap PiP and Fullscreen during recording**
- One click on the toolbar button switches mid-recording. Mac also has Control+Shift+F.
- Platform: Both. Version: Mac 2024.0.0; Win 2024.1.0.
- Sources: [S3][S12][W24][M24][S29]

**Mirror camera**
- Flips the camera horizontally.
- Platform: Mac. Version: 2022.2.0.
- Sources: [S3][M22]

**Camera border**
- A solid-color or gradient border around the PiP camera. Set in Camera Settings on the Video tab.
- Platform: Mac. Version: 2025.4.0.
- Sources: [S3][S1][M25][S29]

**Blur camera background**
- Blurs the background behind the presenter.
- Platform: Mac. Version: 2024.0.0.
- Sources: [S3][M24][S29][S30]

**Virtual backgrounds**
- Options: None, Blur, built-in background images, or a custom uploaded image ("+").
- Lives in a Backgrounds tab in Camera Settings, reachable from the Video tab or the toolbar.
- Not supported on Intel Macs, which get None or Blur only. Marketed as an AI feature.
- Platform: Mac. Version: 2025.0.0; UI refresh in 2025.1.0.
- Sources: [S3][S1][M25][S29][R1]

**Virtual camera devices**
- Win: OBS Virtual Camera is supported (2022.1.1).
- Mac: virtual cameras (XSplit VCam, ManyCam) are listed as unsupported. Tested devices are Logitech Brio and C920/C922/C930 plus built-in cameras. Continuity Camera is referenced.
- Platform: Win / Mac respectively.
- Sources: [W22][K9]

**"Enable webcam" / "Enable Camera" preference**
- Turns webcam support off entirely. Helps performance, and avoids a "camera already in use" error when another app such as Zoom holds the camera.
- Platform: Win.
- Sources: [S10][K4][K15]

**Camera-in-use and permission messaging**
- Win 2026.4.0: clearer message when the chosen camera is busy.
- Mac 2026.3.0: notification if camera options are changed while camera permission is off.
- Platform: Both.
- Sources: [W26][M26][K15]

**Webcam indicator in video**
- A black dot or crossed-camera icon appears in the video when the webcam is enabled but blocked or covered.
- Platform: Win.
- Sources: [K13][K22]

**Citrix webcam support**
- Better webcam recording in Citrix, enabled via a registry flag.
- Platform: Win. Version: 2021.4.3.
- Sources: [W21]

**Webcam is burned in**
- After recording, the webcam overlay cannot be repositioned in Snagit. Separate camera tracks exist only via the Camtasia Editor route (section J).
- Platform: Both.
- Sources: [R2][S4]

---

## F. Audio

**Microphone**
- Toggle plus device dropdown; records from a built-in, USB or analog mic.
- Interfaces and mixers: guidance recommends 44.1 or 48 kHz at 16/24-bit. One article says external audio interfaces are unsupported on Win; another lists tested Focusrite units.
- Bluetooth is not recommended.
- Platform: Both.
- Sources: [S1][K12][K20]

**System Audio**
- Records computer sound (alerts, app and web audio).
- Win: not supported on XP/Vista (legacy note).
- Mac:
  - Before macOS 13, needs a third-party Audio Capture Engine (ACE) plug-in, which requires a security approval for "Rogue Amoeba Software" and, on Apple Silicon, reduced security.
  - macOS 13+ uses native system-audio capture (2022.2.3).
  - Not supported on 10.14 or earlier.
  - Monterey support: 2021.4.3.
  - Left/right channel improvement: 2022.2.0.
- Platform: Both.
- Sources: [S1][S28][K5][K6][M21][M22][M23][S11]

**Remove background noise**
- Live AI noise filtering (fans, AC hum, nearby chatter, keyboard, barking).
- Found under the Microphone options on the Video tab.
- Platform: Both. Version: Mac 2024.3.0; Win (and both-platform launch) 2025.0.0.
- Sources: [S1][M24][W25][M25][S29]

**Automatically adjust input (mic gain leveling)**
- Auto-gain for consistent volume; can be turned off.
- Platform: Mac. Version: 2023.2.0.
- Sources: [S1][M23][M24]

**Combine audio tracks**
- Mixes mic and system audio into one mono track. Turn it off to keep separate tracks for editing in Camtasia.
- Platform: Mac.
- Sources: [S11][M24]

**Audio level meters**
- Live input levels on the mic and system-audio buttons.
- Fixes: Mac 2023.2.0; Win 2026.4.0.
- Platform: Both.
- Sources: [M23][W26][R4]

**Audio format**
- Audio bitrate was raised to 128 kbps stereo.
- Platform: Win. Version: 2019.1.5.
- Sources: [W19]

**Device hot-plug handling**
- Enabling or disabling devices is picked up without a restart (Win 2021.4.0).
- A mic disconnect no longer loses the video (Mac 2022.2.0).
- Platform: Both.
- Sources: [W21][M22]

**Mic + call audio (webinar recording)**
- Record your own mic and the remote party via system audio. Headphones are recommended to avoid echo.
- Platform: Win (documented).
- Sources: [K8][K21]

**Audio editing in Snagit**
- Limited to mute/volume during playback and Make a Silent Copy (Mac, section L).
- Platform: Both.
- Sources: [S5]

---

## G. Cursor, click visualization and keystrokes

**Capture Cursor**
- Toggles whether the pointer is recorded.
- Selecting any cursor effect turns it on automatically (Win 2024.0.3).
- Turning it back on restores the previously chosen effects (Mac 2025.2.0 fix).
- Platform: Both.
- Sources: [S1][W24][M25]

**Highlight Cursor**
- A colored highlight around the pointer while recording. Win: picking a color turns it on.
- Panel sub-options (size, opacity) are not documented (?).
- Platform: Both. Version: Win 2023.2.0; Mac 2023.2.0 (requires macOS 13+).
- Sources: [S1][W23][M23][W24][S30]

**Click Animation**
- A ripple or animation at each click. Color is customizable; toggled from Capture Cursor settings or live from the toolbar.
- Animation sped up in Win 2024.0.1.
- Platform: Both. Version: 2024.0.0.
- Sources: [S1][W24][M24][S29][S30]

**Right-click animation color**
- Secondary clicks can use a different color from left clicks.
- Platform: Mac. Version: 2025.3.0.
- Sources: [M25]

**Capture Hotkeys in Video (keystroke display)**
- Experimental. Logs hotkey combinations and individual key presses and shows them in the final video.
- Enabled in Settings → Experiments, then toggled in the Screen Draw options.
- Platform: Mac. Version: 2024.2.0.
- Sources: [MRN][M24]

**Cursor in Camtasia multitrack**
- The cursor is captured as separate data, so Camtasia Editor can remove, resize or restyle it and edit its path.
- Platform: Both. Version: Mac 2025.3.0; Win 2026.1.0.
- Sources: [S4][S1]

---

## H. Screen Draw (live annotation while recording)

**Screen Draw mode**
- Toggle drawing on the live screen with the Screen Draw button or Ctrl/Control+Shift+D. Shapes fade out automatically.
- While drawing, the user can scroll with the mouse wheel (2023.1.0) but cannot click, type or switch apps.
- Platform: Both. Version: 2023.0.0.
- Sources: [S9][W23][M23][S12]

**Tools**
- Arrow (A), Square (S), Step numbers (1; pressing 1 again restarts numbering).
- Pen (P): Mac only, experimental 2024.1.0, standard 2025.0.0.
- Oval: Mac, experimental 2024.1.0.
- The hotkeys guide also lists Magnifier (M) (?).
- Platform: Both, plus Mac extras.
- Sources: [S9][S12][S34][M24][M25][S29]

**Color**
- A color picker on the Screen Draw toolbar. The palette comes from the active Editor theme or Quick Styles.
- Platform: Both.
- Sources: [S9][W24]

**Fade time**
- Default 3 seconds; customizable in Screen Draw options on the Video tab.
- Objects also fade when leaving Draw mode.
- Platform: Both. Version: 2023.1.0.
- Sources: [S9][S1][W23][M23]

**Screen Draw on/off at capture start**
- A Video-tab toggle decides whether Screen Draw is available.
- Platform: Both (Win note 2023.1.0).
- Sources: [W23][S1]

**Screen Draw always available**
- Experimental: Screen Draw tools can be used at any time.
- Platform: Mac. Version: 2026.3.0.
- Sources: [M26]

---

## I. Where the recording goes after Stop

**Open In (replaced the "Preview in Editor" toggle)**

| Option | Behavior |
|---|---|
| Snagit Editor (default) | Opens the video in Snagit Editor |
| None | Saves to the Library without opening Editor |
| Camtasia Editor | Multitrack hand-off; not saved to the Snagit Library |
| Camtasia Online | Sends video, audio and cursor data online; not saved to the Snagit Library |

- Version: Snagit Editor / None / Camtasia Editor in Mac 2025.3.0 and Win 2026.1.0; Camtasia Online in 2026.4.0 on both.
- Platform: Both.
- Sources: [S1][M25][W26][M26][S20][R3]

**Share destination on the Video tab**
- Pick a destination, or None, that runs automatically after capture.
- Platform: Both.
- Sources: [S1][R4]

**Library storage and autosave**
- Videos are saved as `.mp4` in the Snagit Library (Win `Documents\Snagit`; Mac `~/Pictures/Snagit`).
- Win preference "Automatically save new video captures in the Library": if it is off, unsaved videos are deleted when Snagit closes.
- Since 2023.2.0, videos carry app and website metadata for Library categories.
- Platform: Both.
- Sources: [S10][S11][W23]

**Recent Captures Tray**
- Pin, tag and reorder videos and GIFs; the tray holds 1–200 items.
- Pinning: Win 2024.0.0, Mac 2026.1.0. Video tagging: Win 2024.0.3.
- Platform: Both.
- Sources: [W24][M26][S10]

**Import of external videos**
- Mac: open macOS system recordings (Grab/QuickTime) and AirDrop videos in Snagit (AirDrop: 2024.3.0); `.mov` import without conversion (2025.3.0).
- Win: drag/open MP4.
- Platform: Mac (mainly).
- Sources: [S11][M24][M25][M19]

---

## J. Camtasia hand-off and related

**Record to Camtasia Editor (multitrack)**
- Screen, camera, cursor and audio are recorded as separate streams.
- Opens in Camtasia Rev, which offers Looks/layouts and quick effects. Rev exports to Screencast, YouTube, MP4, GIF, M4A (Win) or MOV (Mac), or "Open in Editor" for the full timeline.
- Exports carry a watermark without a Camtasia Essentials/Create/Pro subscription.
- Platform: Both. Version: Mac 2025.3.0; Win 2026.1.0.
- Sources: [S4][S20][S29][R3]

**Record to Camtasia Online**
- Recording goes straight to the web editor: layouts, effects, animated backgrounds, teammate collaboration.
- Platform: Both. Version: 2026.4.0.
- Sources: [S1][S20][W26][M26]

**Send / Edit in Camtasia (Editor toolbar button)**
- One click sends the open video to Camtasia Editor.
- Multiple files at once: Mac 2025.3.0; Win 2025.4.0.
- Older Mac versions required Camtasia 2021.0.11+ for sharing on Monterey.
- Platform: Both. Version: Win 2024.2.2; Mac 2024.4.0.
- Sources: [W24][M24][W25][S29][M22]

**Share videos to Camtasia Online**
- Upload one or more videos via Share Link → Camtasia Online, combine them into one project, apply Looks and transitions, invite collaborators. Videos arrive as MP4.
- Platform: Both. Version: Mac 2026.1.0; Win 2026.2.0.
- Sources: [M26][W26][S15]

**Images → Camtasia Online narrated video**
- Each image becomes a scene; the user records camera and/or mic per scene.
- Limits:
  - up to 5 minutes of narration per image;
  - images up to 6720×6720 px (3360 at 2× on Mac);
  - scenes without narration display for 0.25–60 s;
  - transitions last 0.1–1 s;
  - Chrome or Edge only; virtual machines are unsupported.
- Exports to Screencast, MP4 download or Camtasia Editor.
- Platform: Both. Version: 2025.4.0.
- Sources: [S8][S21][W25][M25]

**Step Capture → video (Camtasia Editor or Online)**
- Steps become timeline clips or scenes for narration. Requires Snagit 2026.3.0+ and Camtasia Editor 2026.2.0+.
- This is image-adjacent; listed for completeness.
- Platform: Both. Version: 2026.3.0.
- Sources: [S20][S21][W26][M26]

**Edit Images from Camtasia**
- Right-click an image in Camtasia's Media Bin → Edit in Snagit → Send (Win) or Finish (Mac). Needs Camtasia 2025.2.0+.
- Image only; adjacent.
- Platform: Both. Version: Win 2025.2.0.
- Sources: [W25][S20]

---

## K. Video from Images and narration over images

**Create Video from Images**
- Select several images (Ctrl/Cmd+click), then Create → Video from Images or File → Create Video from Images.
- Hotkey Ctrl+Alt+V (Win) / Option+V (Mac).
- Narrate over the images while stepping through them; the result is an MP4 that can also become a GIF.
- Platform: Both. Version: 2020.0.0 (Win hotkey 2020.0.1).
- Sources: [S8][S25][W20][M20][S12]

**Recording toolbar options**
- Video Background Color (fills around the images).
- Cursor capture toggle.
- Microphone and device.
- Camera and device/display (on by default since Win 2022.1.1).
- Record, Pause, Stop using the same hotkeys as screen video.
- Platform: Both.
- Sources: [S8][S25][W22][W25]

**Image navigation and reordering**
- Next/Previous Image buttons or the arrow keys; the Mac hotkeys guide lists Cmd+] and Cmd+[.
- Drag images in the tray to reorder.
- Platform: Both.
- Sources: [S8][S12][W20]

**Live annotation from Favorites**
- During recording or pause, click a Quick Style saved in Favorites and drag on the image. Annotations are added to the images and can be removed afterward.
- Platform: Both.
- Sources: [S8][S25]

**Canvas zoom and dimensions during Video from Images**
- Adjust image zoom and dimensions on the canvas.
- Platform: Win only.
- Sources: [S8][S25]

**Output dimensions**
- 2240×1260 (Win) and 1150×647 (Mac).
- Mac 2026.1.0 turned off Retina downsampling here for higher resolution.
- Platform: Both.
- Sources: [S8][M26]

**Add Narration**
- Record mic narration over one or more tray images to build a slideshow or explainer video.
- Platform: Mac. Version: 2024.4.0.
- Sources: [M24][S29]

---

## L. Video editing in Snagit Editor

**Playback toolbar**
- Elements: elapsed time, total duration, white playhead (drag to scrub), Play/Pause, Previous/Next Frame (hold to rewind or fast-forward), Mute/Unmute with a volume slider, Create Animated GIF, Save Frame as Image, Add Videos.
- Refreshed in 2024.0.0. The controls were made draggable on Mac in 2020.1.4.
- Platform: Both.
- Sources: [S5][W24][M24][M20]

**Video menu**
- Groups playback, editing, combine, Jump To and Make a Silent Copy, with shortcuts.
- Platform: Both. Version: 2024.0.0.
- Sources: [W24][M24][S29]

**Playback speed**
- Review-only speed change; does not affect output.
- Platform: Mac. Version: 2022.2.1.
- Sources: [S5][M22]

**Click to play**
- Clicking the video on the canvas plays, pauses or resumes it.
- Platform: Mac. Version: 2026.1.0.
- Sources: [M26]

**Playback hotkeys**
- Space = play/pause; Left/Right = frame step; Cmd+Return = jump to start; Shift+Cmd+Return = jump to end.
- Platform: Mac (documented).
- Sources: [S12]

**Selection handles**
- Green start and red end handles select a range.
- Handles snap to the playhead (Mac 2024.2.0), and frames preview while dragging them (Mac 2024.0.0).
- Platform: Both.
- Sources: [S5][M24]

**Cut Out (cut and trim)**
- Removes the selected range; a yellow line marks each cut. Trimming the start or end uses the same tool.
- Undo and redo are supported.
- Shared or saved output honors cuts (a Win fix in 2025.4.0).
- Copy All respects cuts (Mac).
- This is the only real video edit Snagit offers; reviewers note there is no other editing.
- Platform: Both.
- Sources: [S5][S2][W24][W25][M19][R2]

**Save As New Video (clip or split)**
- Saves the selected range as a separate video. Repeat to split a long video into several clips.
- Platform: Both. Version: Mac 2024.1.0; Win 2024.2.0.
- Sources: [S5][M24][W24]

**Save Frame as Image**
- Saves the frame under the playhead as a still image, which opens on the canvas (older Mac docs say "Save/Export Frame as PNG").
- Platform: Both.
- Sources: [S5][R4][M21][M22][S30]

**Grab Text from a video frame (OCR)**
- Extracts text from the current frame, during playback or while paused; captures text color since 2022.2.2.
- Platform: Mac. Version: 2022.2.0.
- Sources: [M22][M23]

**Make a Silent Copy**
- Duplicates the video with all audio removed. Partial audio removal is not possible.
- Platform: Mac. Version: 2024.3.2.
- Sources: [S5][M24][S29]

**Get Info shows frame rate**
- The video info dialog includes the frame rate.
- Platform: Mac. Version: 2024.2.0.
- Sources: [M24][S29]

**No annotations or effects on video**
- Annotation tools and effects apply to images only. TechSmith directs users to Camtasia for video annotation.
- The Effects menu is disabled while Cut Out is active (Win 2024.0.0).
- Platform: Both.
- Sources: [R5][W24]

**File → Export for videos**
- Export menu support for video files.
- Platform: Mac. Version: 2025.1.0.
- Sources: [M25]

**Fast handling of edited videos**
- Edits are kept when dragging a video from the tray, and there is no processing delay when sharing or duplicating.
- Platform: Both. Version: Win 2024.1.1; Mac 2024.2.0–2024.2.1.
- Sources: [S29][M24]

---

## M. Combine videos

**Combine Videos**
- Merge two or more videos into one.
- Start it from Add Videos, Video menu → Combine Videos, or by multi-selecting in the tray and right-clicking.
- The dialog lists Library videos sortable by Date Modified, Date Created, Name or Size. A bottom tray shows the order; drag to reorder.
- Unsaved cuts are applied automatically.
- Mismatched sizes get black borders to match the largest video.
- The result opens on the canvas; originals are kept.
- Fixes: a 30 fps cap was removed (Mac 2025.2.0); Quick Look preview with Space (Mac 2024.0.1).
- Platform: Both. Version: 2023.2.0 (window 2024.0.0; reorder tray Win 2024.2.0 / Mac 2024.2.5).
- Sources: [S6][S5][W23][M23][W24][M24][M25]

---

## N. Animated GIF

**Create Animated GIF from video**
- Button on the playback toolbar ("Save As Animated GIF").
- The user chooses the current selection or the whole video.
- Guidance: keep clips under 30 s and files around 1 MB or less.
- The GIF opens on the canvas or at the start of the tray.
- The option is disabled when a GIF is already selected (Mac 2024.1.0).
- Platform: Both. Version: long-standing.
- Sources: [S7][S5][R4][M24]

**"Optimize For" presets**
- Screen Video (default) — for screen recordings.
- High Motion Video — for real-world footage or games.
- Reduced File Size — scales so the largest side is at most 200 px.
- Custom — opens the settings below.
- Platform: Both.
- Sources: [S7][S26][S31]

**Custom GIF settings**
- Frame Rate (fps).
- Dimensions: unit selector (e.g., Percent), Width and Height, Lock Aspect Ratio.
- Enable Dithering (less banding, bigger files).
- Looping: continuous, or play once. The marketing page mentions a set number of repetitions; Mac 2023.2.5 clarified a play-count display (?).
- Dynamic Colors (Mac only).
- Fade to Black at the end.
- Defaults for these are not documented.
- Platform: Both, plus the Mac-only option.
- Sources: [S7][S26][S31][K16][M23]

**Recommended high-quality recipe**
- Custom; 30 fps; 100% width and height; dithering off; looping on; fade optional.
- Platform: Win (documented).
- Sources: [K16]

**GIF from Video from Images**
- Narrated or drawn image sequences can be saved as a GIF.
- Platform: Both. Version: 2020.0.0.
- Sources: [W20][M20]

**GIF engine and housekeeping**
- Much smaller GIFs (Win 2021.0.0).
- Snagit stays usable while a GIF is being created (Win 2020.1.2).
- Cancelling a GIF export no longer crashes (Mac 2023.1.0).
- The giflib library was updated (Mac 5.2.2 → 6.1.3; Win 6.1.3 for CVE-2025-31344).
- Export is disabled when only videos or GIFs are selected (Mac 2026.3.2).
- Platform: Both.
- Sources: [W21][W20][M23][M24][W25][M25][M26]

---

## O. Output formats and sharing

**MP4 output**
- Videos are `.mp4`: H.264 by default; H.265 is optional on Mac.
- `.mov` can be imported on Mac.
- Share Link on Mac accepts MP4 and MOV.
- Platform: Both.
- Sources: [S10][S11][S14][M25]

**Share Link**
- One-click upload with the link copied to the clipboard. Default target is Screencast.
- Admins can redirect it to OneDrive (Win) or Google Drive.
- Can prompt for a file name.
- Disabled when there is no file, several files, or (Mac) an unsupported format.
- Platform: Both. Version: 2023.
- Sources: [S14][W25][M25]

**Screencast (TechSmith hosting)**
- Viewers can comment and react without an account; access can be open or phrase-protected.
- Free: 25 videos, 12-month storage. Pro: unlimited videos plus AI captions, AI titles/descriptions and AI chapters (English only).
- Snagit optimizes videos before upload (Mac 2024.2.3); the optimization can be cancelled (Mac 2024.3.0).
- Shows a notice when the description is over the length limit (Win 2025.1.0).
- Platform: Both.
- Sources: [S33][S14][S30][M24][W25]

**Share destinations that accept video**
- File, Screencast, Camtasia Editor, Camtasia Online, YouTube (resizes to standard sizes), Google Drive, Dropbox, Box (not a Mac default), OneDrive and SharePoint (Win), Slack, Microsoft Teams, PowerPoint (Win 2023.2.1), X/Twitter (Win), FTP, Program (Win) / Application (Mac).
- Mac only: Outlook, Pages, Keynote, Numbers, AirDrop, Mail, Messages, Notes, Photos and others.
- Removed: Knowmia (2025.0.0) and Panopto (Win 2024.3.3; Mac 2026.0.0).
- Platform: Both.
- Sources: [S15][S2][W23][W24][W25][M26]

**Copy All / drag-out / Default sharing format**
- Copy All copies a video for pasting elsewhere.
- Videos can be dragged from the tray into other apps.
- Mac "Default sharing format" governs Export, Copy All and drag; Save As remembers the last format used.
- Platform: Both.
- Sources: [S15][S11][M19]

**Descriptive share titles**
- Shared file names use the source app or page name; can be turned off.
- Platform: Both.
- Sources: [S15][S10][W23]

**AI captions or transcripts in Snagit itself**
- Not found. Captions and chapters exist only as Screencast Pro features applied after sharing; caption editing lives in Camtasia Editor.
- Platform: n/a.
- Sources: [S30][S33][S4][K11]

---

## P. Video-related hotkeys (defaults)

| Action | Win | Mac |
|---|---|---|
| Global capture | Print Screen | Control+Shift+C |
| Video capture (direct) | – | Control+Shift+V |
| Start recording | Shift+F9 | Control+Shift+Space (some pages: Cmd+Shift+Space) |
| Pause/Resume | Shift+F9 | Control+Shift+Space |
| Stop | Shift+F10 | Control+Shift+V |
| Swap PiP / Fullscreen camera | – (button only) | Control+Shift+F |
| Video guides | G | G |
| Cancel | Esc | Esc |
| Fullscreen during selection | – | F |
| Screen Draw on/off | Ctrl+Shift+D | Control+Shift+D |
| Screen Draw Arrow / Square / Step (restart numbering) | A / S / 1 | A / S / 1 |
| Screen Draw Pen | – | P |
| Magnifier (?) | M | M |
| Video from Images | Ctrl+Alt+V | Option+V |
| Next / Previous image | → / ← | Cmd+] / Cmd+[ (tutorial also says arrows) |
| Playback play/pause, frame step, jump to start/end | n/a in docs | Space, ←/→, Cmd+Return, Shift+Cmd+Return |

Hotkeys can be remapped (Win: Capture Preferences → Hotkeys, with an option to override other apps; Mac: Settings → Keyboard). Sources: [S12][S34][S1][S9][S17][S10][S11]

---

## Q. Platform requirements, permissions and limits

**macOS permissions**
- Screen & System Audio Recording for Snagit and SnagitHelper; some changes need Quit & Reopen.
- Settings → Capture → Fix Capture Permissions reopens the dialog.
- Camera and Microphone privacy permissions are required on macOS 10.14+.
- Capture resumes automatically once permissions are granted (2024.1.0).
- Platform: Mac.
- Sources: [S13][S11][R4][M24]

**macOS 15 Sequoia periodic prompt**
- SnagitHelper repeatedly asks to bypass the private window picker. Users should choose "Allow For One Month."
- Platform: Mac. Version: 2024.
- Sources: [K7]

**macOS-version-specific behavior**
- Cursor highlight needs macOS 13+; native system audio needs macOS 13+.
- Tahoe 26 toolbar fixes (2025.3.1); macOS 27 playback and PiP-snapping fixes (2026.3.3, 2026.4.0).
- Platform: Mac.
- Sources: [M23][M25][M26]

**Windows requirements for video**
- Intel 8th gen or Ryzen 2000+ for recording; 8 GB RAM recommended; drivers kept current.
- Windows 10 camera/mic privacy messaging (2019.0.1).
- H.265 files may not play on Windows 10.
- Security tools such as anti-keyloggers can block capture.
- Platform: Win.
- Sources: [K4][W19][M25][K17]

**Protected content**
- HDCP-protected media may not be recordable.
- Platform: Both.
- Sources: [K14]

**Virtual and remote environments**
- Toolbar fix for VDI/RDP (Win 2025.2.0); Citrix webcam flag.
- Platform: Win.
- Sources: [W25][W21]

---

## R. Enterprise and admin controls

**Deploy with video capture disabled**
- Removes video recording; related tours are hidden too.
- Platform: Both. Version: Win 2024.0.3; Mac 2024.4.0.
- Sources: [W24][M24][S29]

**Webcam-on-by-default policy**
- Admins can turn off the default-on webcam.
- Platform: Win. Version: 2022.1.0.
- Sources: [W22]

**Share destination lockdown and Share Link customization**
- Disable destinations; redirect Share Link to OneDrive or Google Drive.
- Platform: Both. Version: 2025.0.0+.
- Sources: [W25][S14]

**Experiments and AI controls**
- Experiments can be disabled via MSI, and the Experiments tab is hidden in analytics-free deployments (Mac 2024.2.0).
- Core on-device AI features cannot be turned off.
- Platform: Both.
- Sources: [K11][S29]

**Update channel**
- Standard or Delayed updates.
- Platform: Both. Version: 2026.4.0.
- Sources: [W26][M26][S10]

---

## S. Searched but not found in public docs

- Keystroke display on Windows: only the Mac experimental "Capture Hotkeys in Video" exists.
- Webcam virtual backgrounds, blur, border or mirror on Windows: all documented as Mac-only.
- In-app AI captions or transcripts in Snagit: Screencast Pro only.
- A recording Timer on Windows.
- H.265 on Windows.
- Exact Restart/Delete button names (?).
- GIF default values.
- Highlight Cursor and Click Animation sub-option lists (size, opacity, shape).
- A per-recording countdown length other than 3 s.

---

## Sources (deduplicated list of URLs consulted)

**TechSmith tutorials and product pages**

| Key | URL |
|---|---|
| S1 | https://www.techsmith.com/learn/tutorials/snagit/how-to-capture-video/ (also via https://www.techsmith.com/learn/wp-json/wp/v2/tutorial/223193) |
| S2 | https://www.techsmith.com/learn/tutorials/snagit/record-edit-share-snagit-video/ (also https://www.techsmith.com/learn/wp-json/wp/v2/tutorial/255238) |
| S3 | https://www.techsmith.com/learn/tutorials/snagit/picture-in-picture/ |
| S4 | https://www.techsmith.com/learn/tutorials/snagit/multitrack-recording/ |
| S5 | https://www.techsmith.com/learn/tutorials/snagit/editing-video/ |
| S6 | https://www.techsmith.com/learn/tutorials/snagit/combine-videos/ |
| S7 | https://www.techsmith.com/learn/tutorials/snagit/animated-gif/ |
| S8 | https://www.techsmith.com/learn/tutorials/snagit/video-from-images/ |
| S9 | https://www.techsmith.com/learn/tutorials/snagit/screen-draw/ |
| S10 | https://www.techsmith.com/learn/tutorials/snagit/preferences/ (also https://www.techsmith.com/learn/wp-json/wp/v2/tutorial/219948) |
| S11 | https://www.techsmith.com/learn/tutorials/snagit/preferences-mac/ |
| S12 | https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/ |
| S13 | https://www.techsmith.com/learn/tutorials/snagit/macos-permissions/ |
| S14 | https://www.techsmith.com/learn/tutorials/snagit/share-link/ |
| S15 | https://www.techsmith.com/learn/tutorials/snagit/share/ |
| S16 | https://www.techsmith.com/learn/tutorials/snagit/presets/ |
| S17 | https://www.techsmith.com/learn/tutorials/snagit/change-global-hotkey/ |
| S18 | https://www.techsmith.com/learn/tutorials/snagit/how-to-capture-your-screen/ |
| S19 | https://www.techsmith.com/learn/tutorials/snagit/get-to-know-snagit |
| S20 | https://www.techsmith.com/learn/tutorials/snagit/whats-new/ |
| S21 | https://www.techsmith.com/learn/tutorials/snagit/step-capture-to-camtasia-online/ |
| S22 | https://www.techsmith.com/learn/tutorials/snagit/snagit-previous-version-help/ |
| S23 | https://www.techsmith.com/learn/tutorials/snagit/ |
| S24 | https://techsmith.com/learn/tutorials/snagit/customize-snagit |
| S25 | https://www.techsmith.com/blog/tutorials/snagit/video-from-images/ |
| S26 | https://techsmith.com/tutorial-snagit-animated-gif.html |
| S27 | https://www.techsmith.com/tutorial-snagit-picture-in-picture.html |
| S28 | https://www.techsmith.com/tutorial-snagit-recording-video.html (legacy) |
| S29 | https://www.techsmith.com/snagit/whats-new/ (pages `?current_page=2` through `?current_page=13` also read; https://www.techsmith.com/?p=308034 resolves here) |
| S30 | https://techsmith.com/snagit/features |
| S31 | https://www.techsmith.com/snagit/features/video-to-gif/ |
| S32 | https://www.techsmith.com/snagit/features/quick-share-screen-recorder/ |
| S33 | https://www.techsmith.com/screencast/ |
| S34 | https://www.techsmith.com/learn/wp-content/uploads/2024/07/Snagit-Shortcuts-Guide-Printable-Mac.pdf |
| — | https://techsmith.com/snagit/whats-new/add-narration-to-your-images (no narration content found) |

**Official version histories**

| Key | URL |
|---|---|
| WIDX | https://support.techsmith.com/hc/en-us/sections/39094961933453 (redirect from https://techsmith.com/snagit-version-history.html) |
| W26 | https://support.techsmith.com/hc/en-us/articles/42674936732685-Snagit-Windows-2026-Version-History |
| W25 | https://support.techsmith.com/hc/en-us/articles/39102293598349-Snagit-Windows-2025-Version-History |
| W24 | https://support.techsmith.com/hc/en-us/articles/39102250364429-Snagit-Windows-2024-Version-History |
| W23 | https://support.techsmith.com/hc/en-us/articles/39102261751949-Snagit-Windows-2023-Version-History |
| W22 | https://support.techsmith.com/hc/en-us/articles/43410048034061-Snagit-Windows-2022-Version-History |
| W21 | https://support.techsmith.com/hc/en-us/articles/43410747342605-Snagit-Windows-2021-Version-History |
| W20 | https://support.techsmith.com/hc/en-us/articles/43410801744525-Snagit-Windows-2020-Version-History |
| W19 | https://support.techsmith.com/hc/en-us/articles/43414049971213-Snagit-Windows-2019-Version-History |
| MIDX | https://support.techsmith.com/hc/en-us/articles/115006443147 |
| M26 | https://support.techsmith.com/hc/en-us/articles/41975263481613-Snagit-Mac-2026-Version-History |
| M25 | https://support.techsmith.com/hc/en-us/articles/37938520706957-Snagit-2025-Version-History |
| M24 | https://support.techsmith.com/hc/en-us/articles/37938485483021-Snagit-Mac-2024-Version-History |
| M23 | https://support.techsmith.com/hc/en-us/articles/37938496753549-Snagit-Mac-2023-Version-History |
| M22 | https://support.techsmith.com/hc/en-us/articles/43415422714125-Snagit-Mac-2022-Version-History |
| M21 | https://support.techsmith.com/hc/en-us/articles/43415439595533-Snagit-Mac-2021-Version-History |
| M20 | https://support.techsmith.com/hc/en-us/articles/43415462314125-Snagit-Mac-2020-Version-History |
| M19 | https://support.techsmith.com/hc/en-us/articles/43415505223437-Snagit-Mac-2019-Version-History |
| MRN | https://download.techsmith.com/update/snagitmac/enu/2420/rnotes.html (Mac 2024.2.0 release notes) |

**Support articles**

| Key | URL |
|---|---|
| K1 | https://support.techsmith.com/hc/en-us/articles/360029108031-Maximum-Record-Length-Snagit-Video |
| K2 | https://support.techsmith.com/hc/en-us/articles/360031042391-Snagit-Mac-Maximum-Record-Length |
| K3 | https://support.techsmith.com/hc/en-us/articles/217552078 |
| K4 | https://support.techsmith.com/hc/en-us/articles/203731518 |
| K5 | https://support.techsmith.com/hc/en-us/articles/360051987832 |
| K6 | https://support.techsmith.com/hc/en-us/articles/360056384711-M1-M2-Chip-and-the-Snagit-System-Audio-Capture-Component |
| K7 | https://support.techsmith.com/hc/en-us/articles/30302332343693-Snagithelper-2024-Is-Requesting-to-Bypass-the-System-Private-Window-Picker-and-Directly-Access-Your-Screen-and-Audio |
| K8 | https://support.techsmith.com/hc/en-us/articles/360054856751-Record-a-Microphone-and-Incoming-Audio-from-a-Webinar-or-Zoom-With-Snagit |
| K9 | https://support.techsmith.com/hc/en-us/articles/11448691888013-Camera-Webcam-and-Virtual-Camera-Compatibility-Snagit-Mac |
| K10 | https://support.techsmith.com/hc/en-us/articles/8777161357709-Snagit-Mac-Temporary-Recording-Location |
| K11 | https://support.techsmith.com/hc/en-us/articles/46075165106957-How-can-I-disable-AI-features-in-Snagit-and-Camtasia |
| K12 | https://support.techsmith.com/hc/en-us/articles/38076254697101-Advanced-Audio-Setup-FAQ |
| K13 | https://support.techsmith.com/hc/en-us/articles/22307543803917-There-is-a-Black-Dot-or-Camera-Icon-on-My-Snagit-Video |
| K14 | https://support.techsmith.com/hc/en-us/articles/41655796209037-How-to-Record-a-Video-with-Snagit |
| K15 | https://support.techsmith.com/hc/en-us/articles/46044114768397-Unable-to-Start-a-Video-Capture-The-Selected-Camera-Is-Already-in-Use |
| K16 | https://support.techsmith.com/hc/en-us/articles/31970928001165-My-Snagit-GIF-is-Blurry |
| K17 | https://support.techsmith.com/hc/en-us/articles/45436420963085-Capture-Doesn-t-Work-or-Screen-Recording-Fails-Snagit-Camtasia |
| K18 | https://support.techsmith.com/hc/en-us/articles/203731398-During-a-Snagit-Video-Capture-the-Audio-Is-Not-Being-Recorded |
| K19 | https://support.techsmith.com/hc/en-us/articles/360001650911-Video-Capture-Is-Blank-or-Only-Shows-Initializing-Screen-In-Snagit-Editor |
| K20 | https://support.techsmith.com/hc/en-us/articles/360035978152-Audio-Quality-Issues-in-Snagit |
| K21 | https://support.techsmith.com/hc/en-us/articles/360017422132-Echo-in-Snagit-Video-Recording-Audio |
| K22 | https://support.techsmith.com/hc/en-us/articles/25842815166989-Webcam-Unavailable-or-Black-in-Snagit |
| K23 | https://support.techsmith.com/hc/en-us/articles/360040026951-Troubleshooting-Video-in-Snagit-on-Windows |
| K24 | https://support.techsmith.com/hc/en-us/sections/200514918-Capture (Mac capture index) |
| K25 | https://support.techsmith.com/hc/en-us/sections/200514858-Capture (Win capture index) |

**Third-party and legacy material**

| Key | URL |
|---|---|
| R1 | https://www.thurrott.com/windows/windows-11/317143/techsmith-releases-snagit-2025-and-camtasia-2025 |
| R2 | https://techradar.com/reviews/techsmith-snagit-2022 |
| R3 | https://www.neowin.net/amp/snagit-202610/ |
| R4 | https://gulfcoast.edu/administration-departments/information-technology-services/staff/documents/snagit-complete-help-guide-2020.pdf |
| R5 | https://www.techsmith.es/blog/wp-content/uploads/2021/07/Snagit-Mac-Help-PDF.pdf (partially readable) |
| R6 | https://www.wcupa.edu/education-socialWork/technologyCenter/documents/SnagitVideoQS.pdf |
| R7 | https://assets.techsmith.com/Docs/pdf-snagit/Snagit_12_GSG.PDF |

**Follow-up suggestions** (search budget is exhausted, so these need a new run):
- Pull the cursor-effects sub-options and any official naming of Restart and Cancel. Good starting points are the "Cursor effects options" screenshots on [S1] and the live tutorial for Snagit 2026.
- Check whether the four 2026.4.0 features (Timer, Record to Camtasia Online, camera border, background options) have reached Windows.