# Snagit image capture: feature inventory from public sources (Windows and macOS)

**Scope:** image capture only, as Snagit stood in October 2026. That means Snagit 2026.4.0 on Windows (6 Oct 2026) and on Mac (7 Oct 2026). Video recording, Editor, Library and sharing features appear only where they directly affect capture.

**Method:** I used only public TechSmith tutorials, support articles, version histories, release notes, public help PDFs and COM API guides, plus a few press and review pages. No Snagit binary, installer, package or .snagx file was downloaded or inspected. One `curl` of a public help PDF was blocked by the proxy, so I read that document through WebFetch instead. All descriptions are paraphrased.

**Open gaps:** I used up the web-search budget for this turn near the end. The items marked (?) and the gaps in section 16 still need checking in a follow-up pass.

**Conventions**
- Platform is Win, Mac, Both or Unknown.
- "Ver" is the version where a feature was introduced or changed, when a source says so.
- Source keys in [brackets] map to the URL list in section 17.

---

## 1. Ways to start a capture and the capture interface

### 1.1 Snagit Capture background app (tray / menu bar)
- **What it does:** A capture process runs in the background and owns the global hotkeys. The user opens the Capture Window from:
  - Win: the taskbar or notification-area icon.
  - Mac: the menu bar icon.
  - Either: the Capture button inside Snagit Editor.
- **Win:** a preference starts it when Windows starts. Hotkey Ctrl+Shift+X shows/hides or exits Snagit Capture.
- **Mac:** a setting keeps Snagit running in the menu bar and starts it at login.
  - The menu bar icon turns red while recording. A setting can force a black-and-white icon.
  - Right-clicking the menu bar icon opens a quick menu that includes All-in-One capture (Mac 2024.3.0).
- **Tray icon actions (Win):** Step Capture can be paused from the tray icon (crash fix in 2025.3.1). While an Interval or Scheduled capture runs, a menu item changes to "Stop Capture" (fix in 2026.1.0; which menu this is was not stated (?)).
- **Platform:** Both. **Sources:** [T-CAP][T-PREFW][T-PREFM][VH-M24][VH-W25][VH-W26][T-HK]

### 1.2 Capture Window
- **Tabs:** All-in-One, Image, Video, plus a **Presets** list.
- **Controls:**
  - A big **Capture** button starts a capture with the current tab's settings.
  - A **Shortcut** field (Win label "Add Hotkey", Mac "+ Shortcut") sets the global hotkey.
  - A settings gear: Mac gear > More Settings opens Settings. Win uses File > Capture Preferences.
  - An expand/collapse arrow shows or hides the extra All-in-One options.
- **Menus (Win):**
  - File > Capture Preferences
  - File > Import Presets
  - View > Show Capture Widget
  - Help > Check for updates
  - A reset-to-defaults action resets all capture settings. On Mac it also resets the Selection field (fix 2025.0.0) and the Time Delay (fix 2021.2.0).
- **Remembering state:** The window remembers the last selected tab (Mac fix 2025.1.0).
- **Version notes:**
  - Win 2025.4.0: right-clicking the title bar gives the system move/size menu.
  - Accessibility work: hover states (Win and Mac 2025), better keyboard navigation of Presets (Win 2025.3.1) and Time Delay (Win 2026.3.1), refreshed colours and icons (Mac 2026.0.0).
  - Capture UI refreshed in 2021.3.0.
- **Platform:** Both. **Sources:** [T-CAP][T-CAPJ][T-PRE][VH-W25][VH-W26][VH-M25][VH-M26][VH-W21][VH-M21]

### 1.3 Capture Widget (formerly the "OneClick" tab)
- **What it is:** A slim tab docked at a screen edge. Hovering over the blue bar expands it to show a **Capture** button and a **View Presets** icon. It can be dragged along the top or side edges.
- **How to turn it on:**
  - Win: View > Show Capture Widget, or Capture Preferences > General, choosing Capture Window, Capture Widget, or both.
  - Mac: Settings > General > Appearance lists Capture Window, Capture Widget, or both.
- **Fixed defaults for its Capture button** (they ignore the Capture Window settings):
  - Type: All-in-One
  - Preview in Editor: on
  - Capture Cursor: on
  - Delay: none
  - Effects: none
  - Share: none
- **Presets:** View Presets runs a preset by clicking its name. New-preset notifications show in the widget (truncation fix Win 2025.1.0).
- **Version notes:** Renamed from OneClick around 2020–2021 (?). Reliability fixes in Win 2024.3.3 and 2025.1.1.
- **Platform:** Win documented. Mac has the Appearance option (?). **Sources:** [T-PREFW][T-PREFM][KB-MODES][KB-WIDGET][T-CUST][PDF-H21][PDF-W18][PDF-H19][VH-W25]

### 1.4 Global capture hotkey ("Capture" / "Global Capture")
- **What it does:** Starts a capture with whatever the Capture Window currently shows (All-in-One, Image or Video tab and their settings).
- **Defaults:** Win **Print Screen**. Mac **Control+Shift+C**.
- **How to change it:** Click the Shortcut field in the Capture Window and press a new combination. It is also editable in the Win Hotkeys tab and the Mac Keyboard tab.
- **Print Screen conflicts on Windows** (from support articles):
  - Windows 11 can bind Print Screen to Snipping Tool. Snagit 2023.2.1+ handles this; otherwise turn off the Windows accessibility setting that sends Print Screen to the screen-capture tool.
  - Other apps (OneDrive, Dropbox) may own the key. From Snagit 13.0.2, Snagit asks whether to take it over. The Hotkeys-tab toggle "Allow Snagit to override other hotkey assignments" does the same. The override lasts only while Snagit is running.
  - Logitech Options+ key remaps and laptop Fn-lock can also break it.
  - A fix in 2024.0.0 / 2023.2.1 made the override work against Windows 11's Print Screen binding.
- **Platform:** Both. **Sources:** [T-GHK][T-CAP][T-HK][KB-PRTSC][VH-W24][VH-W23]

### 1.5 Repeat Last Capture
- **What it does:** Repeats the previous capture with the same settings.
- **Hotkey:** Mac Control+Shift+R by default. Win has no default; assign one in Capture Preferences > Hotkeys (modifier keys plus a key), or choose "None".
- **Conflicts:** If another app already owns the combination, Snagit reports the hotkey is in use.
- **Platform:** Both. **Sources:** [T-COMMON][T-HK][KB-RLC][PDF-H24]

### 1.6 Opening OS screenshots in Snagit
- **Win:** the preference "Open screenshots copied from system tools" opens Snipping Tool / Snip & Sketch captures in the Editor (Win 2019.1.0). A duplicate-image fix shipped in 2026.3.0.
- **Mac:**
  - "Open system screenshots and videos in Snagit" moves macOS captures into the Library, with a sub-option **Keep original file** (2019.1.1).
  - "Open AirDrop images and videos in Snagit".
  - Earlier macOS Grab captures were auto-imported.
  - Screenshots taken with the system tool during a Snagit recording are imported the next time the Editor opens (Mac 2025.1.0).
- **Platform:** Both. **Sources:** [T-PREFW][T-PREFM][VH-W19][VH-M19][VH-M25][VH-W26]

### 1.7 Capture an entire webpage by dragging its URL (Mac)
- **Three ways to start:**
  - Drag the site icon or URL from the browser address bar onto the Snagit menu bar icon.
  - Drag it onto the Dock icon.
  - Image tab > Selection > **Webpage** > Capture > enter the URL > click **Scroll**.
- **Result:** Snagit captures the full page length, including content that is off screen.
- **Version notes:** Works in Brave and Edge since Mac 2022.1.0. A fix for dropping a URL on the menu bar icon shipped in 2021.1.0.
- **Old caveat (?):** A 2021-era article says web capture worked only for HTTP pages, not HTTPS. This is likely outdated.
- **Platform:** Mac. **Sources:** [T-COMMON][KB-WEBMAC][KB-SCRFAIL][PDF-H20][VH-M22][VH-M21]

### 1.8 Snagit Printer (Print to Snagit)
- **What it does:** Installs a virtual printer named "Snagit <year>". Printing from any application sends the pages into the Editor as an image or multi-page capture.
- **Setup:** File > Capture Preferences > Capture tab > **Printer Capture Settings**. On the Snagit Printers tab, pick the printer or "Add a Snagit Printer"; **Properties** sets the output.
- **Recommended uses:** PDFs and other content that automatic scrolling cannot handle.
- **Version notes:** Page-navigation fix in 2025.3.0.
- **Platform:** Win. **Sources:** [T-COMMON][T-PREFW][PDF-H22][PDF-H21][PDF-H20][KB-NOARROW][VH-W25]

### 1.9 COM Server automation API
- **What it is:** A scripting interface for automating image and video capture. It is the most precise public statement of the underlying capture model.
- **Image inputs:**
  - Desktop (full screen)
  - Window
  - Region
  - Graphic file
  - Clipboard
  - Menu
  - Object (a user-picked on-screen element)
  - Freehand
  - Extended Window (window resized beyond the screen before capture)
  - All-in-One
  - Removed over time: TWAIN scanner/camera (removed 21.0.1); Custom Scroll, link capture and SWF (removed 13.0); DOS screen, program file, ellipse, rounded-rectangle, triangle, polygon and wallpaper shapes, DirectX (removed 11.0).
- **Image outputs:** none, printer, file, clipboard, email, FTP.
- **Window selection:** interactive, active window, by window handle, or the window at an X/Y point.
- **Region selection:** interactive or fixed size.
- **Auto-scroll:** none, vertical, horizontal or both. Start position: current, top, left or top-left. A configurable wait before scrolling.
- **Delay:** on/off, seconds (default 10), countdown window on/off.
- **Other properties:** IncludeCursor (default on), magnifier (default on), multiple-area mode, preview in Editor (default off).
- **File naming:** prompt the user; a fixed name; or automatic with a prefix (default "SNAG-") plus a zero-padded counter (default 4 digits), with a separate counter per file type.
- **Image formats:** BMP, TIFF, JPEG, GIF, PNG (default), TGA, RAS, WMF, EPS, OS/2 BMP, WFX, EMF, WPG, PSD, ICO, CUR, PDF, SNAG, MHT, SNAGX (added 22.0.1).
- **States and events:** idle, succeeded, failed, busy. Events for error, state change and file written.
- **Platform:** Win. **Ver:** All-in-One input since COM 10.0. **Sources:** [COM12][COM25][VH-W20]

---

## 2. All-in-One capture

### 2.1 All-in-One capture flow
- **Steps:**
  1. Start from the All-in-One tab (Capture button or hotkey). Mac also has a dedicated All-in-One hotkey, Control+Shift+O.
  2. Orange crosshairs appear with a magnifier.
  3. Choose the target: drag a region, hover to auto-detect a window (orange dashed outline) and click, or click the **Fullscreen** button.
  4. The All-in-One toolbar appears with **Image Capture**, **Video Capture** and **Scrolling Capture**. On Mac the single keys I, V and S pick these.
  5. Image Capture opens the result in the Editor, or sends it wherever the settings say.
- **Inside the selection:** Scroll arrows appear over scrollable windows for automatic scrolling capture (see 6.2).
- **Adjusting the selection:** Drag handles to resize, drag inside to move, type exact Width/Height pixel values, or drag the toolbar outside the selection so it is not captured.
- **Recovering from mistakes:**
  - A redo control lets the user re-pick; on Mac it remembers the chosen capture type (2021.4.2).
  - A video capture started by mistake can be switched to an image capture (2021.3.0).
- **Platform:** Both. **Ver:** All-in-One debuted in Snagit 10 (Win, 2010). **Sources:** [T-CAP][T-CAPJ][T-SCR][PDF-H21][PDF-H20][PDF-W18][EXT-CAMPUS][T-HK][VH-W21][VH-M21]

### 2.2 All-in-One basic settings (expandable panel)
- **Selection:** Region, Window (Win only per the tutorial), Fullscreen.
- **Share:** a destination, or File (saves PNG/MP4; the gear sets file name and location).
- **Preview in Editor:** on/off.
- **Copy to Clipboard:** on/off.
- **Capture Cursor:** on/off.
- **5 Second Delay:** a fixed 5-second countdown in the bottom-right corner, meant for menus.
- **Version notes:** All-in-One Share only lists destinations that accept both images and video (older help). All-in-One no longer rounds sizes to an even number of pixels (Win 2024.1.2).
- **Platform:** Both. **Sources:** [T-CAP][T-CAPJ][PDF-H21][VH-W24]

---

## 3. Crosshairs and selection behaviour (all region-based modes)

### 3.1 Crosshairs and magnifier
- **Behaviour:** Full-screen orange crosshairs, plus a magnifier window for pixel-accurate selection.
- **Keys:** **M** hides or shows the magnifier during capture. A preference "Show magnifier" sets the default (Win Capture tab).
- **Performance:** Crosshairs appear up to 5x faster after starting a capture (Mac 2025.2.0). Smoother movement in Mac 2021.0.0. Region performance improved in Mac 2020.1.2 and 2024.4.1.
- **Platform:** Both. **Sources:** [T-IMG][T-PREFW][T-HK][VH-M25][VH-M21][VH-M20][VH-M24]

### 3.2 Simplified crosshairs
- **What it does:** A lighter crosshair rendering that improves performance, especially across multiple monitors. Set at Capture Preferences > Capture tab.
- **Related support guidance:**
  - Snagit supports about 2–3 monitors, or two 4K monitors at 60 Hz or less, before crosshairs become sluggish.
  - A registry switch forces software rendering.
- **Platform:** Win. **Ver:** Snagit 12+. **Sources:** [T-PREFW][KB-SLUG]

### 3.3 Window and object auto-detection ("snapping")
- **Behaviour:** Hovering highlights the window, or a sub-area such as a browser's content pane, under the cursor with an orange dashed border; clicking captures it.
- **Mac 2013 review notes:** The selection fits to the window or the browser content area.
- **Known failures:** Overlay software such as NVIDIA GeForce Experience, AMD Adrenalin, Dell Display Manager, Remote Desktop, AnyDesk and TeamViewer can break detection (Win).
- **Version notes:**
  - Detection with screen-sharing apps and Camtasia improved (Win 2023.1.0).
  - Snapping fixed with screen-sharing apps (Mac 2024.4.4).
  - Fixed: detected windows missing a portion (Mac 2024.4.1).
  - Fixed: the cursor being detected as a selection target (Mac 2023.1.0).
  - Fixed: duplicate select areas when sharing in Slack (Win 2021.4.5).
- **Platform:** Both. **Sources:** [T-IMG][PDF-H23][KB-AUTODET][EXT-MACWORLD][EXT-INDEZINE][VH-W23][VH-M24][VH-M23][VH-W21]

### 3.4 Keyboard control during selection

| Action | Win | Mac |
|---|---|---|
| Cancel capture | Esc | Esc |
| Show/hide on-screen capture help | F1 | H |
| Toggle magnifier | M | M |
| Fullscreen capture | Fullscreen button | F, or the Fullscreen button |
| Nudge crosshair 1 px | Arrow keys | Arrow keys adjust the region (Mac fix 2020.2.2) (?) |
| Lock square (1:1) | hold Shift | hold Shift (2019 help) (?) |
| Lock 16:9 | hold Ctrl | hold Shift for video regions per 2022 help (?) |
| Lock 4:3 | hold Ctrl+Shift (older help: Shift for video) | n/a |
| Take capture | Enter | Return (fix 2024.4.2: it had triggered fullscreen) |
| Start a region selection | Shift+Enter | — |
| Toggle standard-size guides | G | G |
| End a multiple-area capture | right-click | release Command |

- **Standard-size guides (G):** Faint guides appear for 1920×1080, 1280×720, 640×480 and 320×240. The border highlights when the selection matches one. They are documented mainly for video; Mac added size guides for image capture in 2021.1.1.
- **Keyboard size editing:** Fixed on Mac Monterey in 2021.4.5 / 2022.0.1.
- **Platform:** Both. **Sources:** [T-HK][PDF-SCW][PDF-H19][PDF-13][PDF-W18][PDF-H21][PDF-H22][VH-M21][VH-M20][VH-M24]

### 3.5 Multi-monitor and DPI
- **Windows:**
  - The **Fullscreen** selection type captures all connected displays together.
  - To capture one monitor, use Region mode and click the Fullscreen button shown at the top centre of that monitor.
  - Mixed-DPI multi-monitor support added in 2019.1.2.
- **Mac:** Fullscreen captures one screen at a time.
- **Platform:** Both, with different behaviour. **Sources:** [KB-MULTI][T-COMMON][PDF-H24][PDF-13][VH-W19][KB-SLUG]

### 3.6 HDR-aware capture
- **What it does:**
  - Win 2024.2.2 added HDR image capture, with a "Support HDR color settings" preference. HDR is not supported for scrolling or video captures.
  - White areas on HDR are more accurate (Win 2024.3.2 / 2025.1.0).
  - Portrait-orientation HDR fixes in 2025.1.0.
  - An experimental "HDR Capture Fidelity" option (Win 2025.4.0; Editor Preferences > Experiments) matches HDR display colours more closely.
- **Platform:** Win. **Sources:** [VH-W24][VH-W25][KB-HDR][T-PREFW][EXT-NEO2540][RN-W25]

---

## 4. Image tab settings

### 4.1 Selection dropdown (capture type)

**Current Windows list:**
- Region
- Window
- Fullscreen
- Scrolling
- Grab Text
- Menu
- Steps (2025+)
- Advanced submenu: Freehand, Multiple Area, Menu
- Also documented as types that skip the drag step: Clipboard, File

**Legacy Windows types (Snagit 13 era):**
- Scrolling Window
- Panoramic
- Object
- Advanced > Scanner/Camera (TWAIN)

**Current Mac list:**
- Region
- Fullscreen
- Scrolling
- Grab Text
- Menu
- Mission Control
- Webcam
- Webpage
- Steps
- Window (added Mac 2026.0.0)
- Multiple areas are done with Command-drag in All-in-One rather than a list entry.

**Behaviour notes:**
- Choosing Grab Text greys out incompatible settings.
- Fixed in Win 2025.0.0: Region-only options appearing for other types.

**Platform:** Both. **Sources:** [T-IMG][T-COMMON][PDF-H24][PDF-H23][PDF-H22][PDF-13][T-STEP][VH-M26][VH-W25]

### 4.2 Selection Properties: Fixed Region and Set Position
- **Steps:** Image tab > Selection = Region > **Selection Properties** > **Fixed region**.
- **Settings:**
  - **Width** and **Height** in pixels.
  - Optional **Set position**: type **X** and **Y**, or click **Select Region** to drag the area on screen.
- **Behaviour:**
  - Without a set position, a box of that size follows the cursor and a click places it.
  - With a set position, Snagit captures those exact coordinates immediately.
- **Typical use:** Saved as a preset for repeated same-area captures, for example 1080×1080.
- **Version notes:**
  - Mac added Fixed Region in 2020.2.0, with workflow tweaks in 2020.2.1.
  - Fixed on Mac: the Capture Window staying visible while picking a fixed region (2023.2.2); snapping highlights against region presets (2021.4.0).
  - Fixed on Win: a crash with very large fixed dimensions (2019.1.2).
- **Platform:** Both. **Sources:** [T-COMMON][PDF-H23][PDF-H22][KB-FIXED][BLOG-HIDDEN][VH-M20][VH-M23][VH-M21][VH-W19]

### 4.3 Effects applied at capture time
- **How it works:** Pick one or more effects in the Image tab's Effects dropdown; they are applied automatically to every capture.
  - "Add another effect" stacks them.
  - "Remove this effect" (Win) or "Remove" (Mac) drops the last one.
  - "None" clears all.
  - Some effects have a gear for options.
- **Effects documented for capture time:**
  - **Capture Info** (see 12.2)
  - **Border**
  - **Corner Rounding** with a gear to set the radius (Mac 2025.2.1)
  - **Scale Image** (Win; recommended for 4K Step Captures)
  - Scale and rotate as capture-preset effects (Mac 2021.1.0)
- **Other Editor effects (?):** Edges, Shadow, Perspective, Page Curl, Watermark, Filters and others exist in the Editor. Whether every one of them can be chosen at capture time is not confirmed.
- **Clipboard copies:** Copy to Clipboard includes capture-time effects (Mac 2020.1.2).
- **Platform:** Both. **Sources:** [T-IMG][PDF-H22][T-FX][T-CI][T-STEP][VH-M25][VH-M21][VH-M20]

### 4.4 Share (destination after capture)
- **What it does:** Sends each capture automatically to one or more destinations. The Image tab lists only image-capable destinations.
  - **Preview in Editor on:** the user clicks **Finish** in the Editor to send.
  - **Preview in Editor off:** the capture goes straight to the destination.
- **File destination:** Its gear sets the file name, format and folder.
  - Mac takes the file name and folder from the Share tab and the JPG/PNG format from Advanced > Default sharing format.
  - Win has Save As Preferences: remember last format, or always use a chosen format.
  - The exact naming options in the dialog are not publicly documented (?). The COM model suggests prompt, fixed, or prefix plus counter.
- **Other destinations:**
  - Win only: Clipboard, Printer, Email, Program, OneDrive, SharePoint, Evernote, X.
  - Mac only: Application, Pages, Keynote, Numbers, Outlook.
  - Both: Screencast, Camtasia Online / Camtasia Editor (2025–2026).
- **Mac extras:** Destinations can have custom names and shortcuts.
- **Version notes:** Fixes for sending to several destinations at once (Win 2021.4.0 and 2025.3.0).
- **Platform:** Both. **Sources:** [T-IMG][T-SHARE][T-PRE][T-PREFW][T-PREFM][KB-SAVEFMT][COM25][VH-W21][VH-W25][VH-M19]

### 4.5 Preview in Editor
- **On:** each capture opens in Snagit Editor.
- **Off:** the capture goes only to Share and Clipboard, and still auto-saves to the Library if that preference is on.
- **Version notes:**
  - Win: a related Editor preference keeps the Editor running in the background for faster opening.
  - Captures with preview off now load in order (Mac fix 2022.2.5).
- **Platform:** Both. **Sources:** [T-IMG][T-CAP][T-PREFW][VH-M22]

### 4.6 Copy to Clipboard
- **What it does:** Puts the capture on the system clipboard right away. Combined with Preview off, this gives a fast "capture to clipboard" workflow.
- **Version notes:**
  - Fixed: cursor missing from clipboard copies with preview off (Mac 2024.4.1).
  - Fixed: wrong cursor size on Retina clipboard copies (Mac 2024.4.5).
  - Fixed: ellipse, polygon and freehand selections pasting as rectangles (Win 2026.0.0).
- **Platform:** Both. **Sources:** [T-IMG][T-COMMON][VH-M24][VH-W26]

### 4.7 Capture Cursor
- **What it does:** Includes the mouse pointer. In image captures the cursor is stored as a separate vector object that can be moved, resized or deleted later.
- **Version notes:**
  - Fixes in Win 2025.0.0 and 2025.2.0, including the cursor object picking up background pixels.
  - Fixed: the cursor in the wrong position (Mac 2024.4.1).
  - Fixed: a large system pointer always being captured (Mac 2023.1.0).
  - All-in-One fullscreen honours the setting (Mac 2020.0.1).
- **Platform:** Both. **Sources:** [T-IMG][T-CAP][VH-W25][VH-M24][VH-M23][VH-M20]

### 4.8 Time Delay (Delay, Interval, Schedule)
- **Delay:** Waits N seconds, with a countdown in the bottom-right corner, then starts the capture. The countdown can be cancelled (Win 2022.1.0).
- **Interval (time-lapse):**
  - Captures the chosen area repeatedly at a set interval.
  - Optionally discards a capture identical to the previous one.
  - The Capture button becomes **Start/Stop**.
  - Minimum interval raised from 1 s to 3 s (Win 2026.3.1).
- **Schedule:** Captures at a set date and time; the button becomes Start/Stop. Captures scheduled less than a minute ahead now start exactly on the minute (Mac 2026.3.2).
- **Edge cases:**
  - For Fullscreen, Menu, Clipboard and File selections, Interval and Schedule need no drag.
  - Presets can hold interval settings; the preset hotkey field stays visible for these toggle-style presets (Win 2021.4.0).
- **Version notes:**
  - Schedule and Interval were documented as Windows-only through the 2024 help; Mac gained both in **Mac 2026.1.0**.
  - Fixed: delayed capture in OneClick (Mac 2021.4.4).
- **Platform:** Both. **Sources:** [T-IMG][T-COMMON][PDF-H24][PDF-H22][VH-W22][VH-W26][VH-M26][VH-W21][VH-M21][RN-W26]

---

## 5. Basic capture modes

### 5.1 Region
- **What it does:** Drag a rectangle with the crosshairs. Supports the magnifier, aspect-ratio and square locks, pixel nudging, handles, typed Width/Height, and Fixed Region (4.2).
- **Platform:** Both. **Sources:** [T-IMG][T-COMMON][T-HK]

### 5.2 Window
- **Windows:** Selection = Window. Hover until the orange dashed outline appears, then click. Captures a window, dialog, or an area inside an application.
- **Mac:**
  - A dedicated "Exposé (Window) Capture" hotkey, Control+Shift+W.
  - Since **Mac 2026.0.0**, Window is in the Selection dropdown for both image and video. For video, recording follows the chosen window when it moves or resizes and excludes other windows.
  - Mac window captures include the window border and shadow (restored on macOS 26 in 2026.1.0).
- **Rounded corners:** Snagit 10 introduced transparency so rounded windows keep their shape. Windows 11 rounded corners are a known current limitation: they come out square.
- **Platform:** Both. **Sources:** [T-IMG][T-COMMON][T-HK][VH-M26][KB-ROUND][EXT-CAMPUS]

### 5.3 Fullscreen
- **What it does:** Captures the whole display (see 3.5 for multi-monitor differences). Available as a selection type and as the Fullscreen button or F key during capture.
- **Experimental (Mac 2025.0.0):** Each window in a fullscreen capture becomes a separate, editable layer.
- **Platform:** Both. **Sources:** [T-IMG][KB-MULTI][VH-M25][RN-M25]

### 5.4 Mission Control capture
- **What it does:** Selection = Mission Control, or hotkey Control+Shift+M. Snagit spreads out all open windows, including hidden or covered ones; hover and click to capture one at full size.
- **Version notes:** Fixes for overlapping windows on Sequoia (2024.4.1) and blurriness on Tahoe (2024.4.7 / 2023.4.0).
- **Platform:** Mac. **Sources:** [T-COMMON][T-HK][PDF-H20][EXT-MACWORLD][VH-M24][VH-M23]

### 5.5 Menu capture
- **Windows:**
  - Selection = Advanced > Menu (or Menu).
  - **Menu Properties:** "Capture opened sub-menus" includes open flyouts; "Capture menu bar" includes the menu's title in the menu bar.
  - Normally paired with Time Delay: open the menu during the countdown, the screen freezes, then crosshairs or menu detection capture it.
- **Mac:**
  - Selection = Menu, or hotkey Control+Shift+E (open the menu, then press the hotkey).
  - Menus are captured on a transparent background (2013 review).
- **Version notes:**
  - Fixed: duplicated menus (Mac 2024.4.0).
  - Fixed: dark menus on Tahoe (2024.4.7).
  - Open menus no longer block capture on Mojave (2019.1.0).
- **Platform:** Both. **Sources:** [T-COMMON][KB-MENU][T-HK][EXT-MACWORLD][VH-M24][VH-M23][VH-M19]

### 5.6 Freehand
- **What it does:** Selection = Advanced > Freehand. The cursor becomes scissors; drag any shape and release to capture it. The area outside the shape is presumably transparent (?).
- **Platform:** Win. **Sources:** [T-COMMON][T-IMG][PDF-13]

### 5.7 Multiple Area capture
- **Windows:** Selection = Advanced > Multiple Area. Add windows (hover and click) or regions (drag) repeatedly, then click **Finish** at the top of the screen or right-click to end.
- **Mac:** In All-in-One, hold **Command** and drag several regions; releasing Command finishes.
- **Result:** One combined image. The layout rules are not documented (?).
- **Version notes:** Crash fix for multi-region captures (Mac 2022.0.1). COM exposes a multiple-area flag.
- **Platform:** Both. **Sources:** [T-COMMON][T-HK][PDF-H22][COM25][VH-M22]

### 5.8 Webcam still image
- **What it does:** Selection = Webcam. Capture opens a dialog with a **Camera** picker (built-in or external) and a Capture button.
- **Windows equivalent:** None documented (?).
- **Platform:** Mac. **Sources:** [T-COMMON][PDF-H20][EXT-MACWORLD]

### 5.9 Clipboard as capture source
- **What it does:** Selection = Clipboard turns current clipboard contents (image, or text rendered to an image (?)) into a capture without a screen pick.
- **Version notes:** Fixed in 2026.4.0 to include all clipboard text.
- **Related Editor path:** File > New from Clipboard.
- **Platform:** Win (Mac unknown). **Sources:** [T-COMMON][PDF-H24][VH-W26][PDF-13][KB-EXCEL]

### 5.10 File as capture source
- **What it does:** Selection = File brings a file's content in as a capture (legacy wording "capture the content of a file"). The current interface is thinly documented (?).
- **Platform:** Win. **Sources:** [T-COMMON][PDF-13][COM25]

### 5.11 Object capture (legacy (?))
- **What it does:** Captured a single on-screen user-interface element. Listed in the Snagit 13 help and still in the 2025 COM API. Its presence in today's dropdown is unconfirmed.
- **Platform:** Win. **Sources:** [PDF-13][COM25]

### 5.12 Scanner / camera import (TWAIN), legacy
- **What it did:** Imported from TWAIN devices. Removed from COM in 21.0.1. Older help listed it as a capture-time mode.
- **Platform:** Win (legacy). **Sources:** [PDF-13][PDF-H21][COM25]

### 5.13 Continuity Camera import
- **What it does:** Imports a photo or scan from an iPhone or iPad via Image > Import from iPhone or iPad.
- **Platform:** Mac. **Ver:** 2025.2.0. **Sources:** [VH-M25]

---

## 6. Scrolling capture

**Naming history:**
- "Panoramic" capture was introduced around Snagit 13 / Mac 4 (2016).
- In **2023.2.0 (both platforms)** it was renamed **Scrolling Capture** (manual).
- The old arrow-based "Scrolling Capture" became **Automatic Scrolling Capture**.

**Sources:** [VH-W23][VH-M23][KB-SCRINC][KB-SCRFAIL][NEWS2016]

### 6.1 Scrolling Capture (manual, formerly Panoramic)
- **Steps:**
  1. In All-in-One, drag a selection inside a window and choose **Scrolling Capture** on the toolbar.
  2. If the preference "Require Start button for scrolling capture" is on, click **Start**; otherwise capture begins at once.
  3. Scroll vertically, horizontally or both, using the scrollbars, mouse wheel or dragging.
  4. A live preview shows the stitched image.
  5. Click **Done** (formerly Stop).
- **If the toolbar is hidden:** Click the Snagit Capture taskbar icon (Win).
- **Processing:** A progress bar and status messages appear (Mac 2023.1.0; Win 2023.2.x). The user can cancel processing (fixed in Win 2023.2.0).
- **Stitching:** Fixed headers, footers and sidebars are handled smarter (2023.2.0).
- **Hiding Snagit from the result:** The preview and controls are removed from the final image when "Hide Snagit during capture" is on (Win 2024.1.0, needs Windows 10 20H1+; Mac fixes 2024.2.5 and 2024.4.6).
- **Tips:**
  - Enlarge the window.
  - Exclude scrollbars, margins and ads.
  - Turn off floating toolbars.
  - Scroll steadily, in one direction at a time.
  - Prefer scrollbars to the wheel, to avoid hover effects.
- **Limits:** Very narrow regions and very small areas once crashed (fixes Win 2023.1.0 and 2026.2.0). Parallax sites are unreliable. HDR is not applied (Win). The scrolling preview no longer minimizes incorrectly (Win 2023.2.2).
- **Version notes:** Long captures save to PDF with better pagination (Mac 2026.4.0). Crash on stop fixed (Win 2026.3.0). Virtual-machine freeze fixed (Win 2026.1.0).
- **Preset:** A built-in "Scrolling Capture" preset ships on Windows and can take a hotkey. On Mac, create a preset with Selection = Scrolling.
- **Platform:** Both. **Sources:** [T-SCR][PDF-H24][PDF-H23][PDF-H21][KB-SCRPREV][T-PREFW][T-PREFM][T-PRE][VH-W23][VH-M23][VH-W24][VH-M24][VH-W26][VH-M26]

### 6.2 Automatic Scrolling Capture (scroll arrows)
- **How it works:** During an All-in-One or Image capture, when the cursor snaps to a scrollable window, animated arrows appear: one for horizontal, one for vertical, one for both or the entire area. On Mac they are yellow and bounce (2023.1.0). Clicking an arrow makes Snagit scroll and stitch by itself.
- **Keys:**
  - Win: H = horizontal, V = vertical, B = both.
  - Mac: R = across, D = down, X = both.
  - The keys are useful in Mac Mail, where the arrows show only over the subject line.
- **Requirements and permissions:**
  - Mac: Accessibility permission for Snagit and SnagitHelper (Settings > Capture > Fix Capture Permissions > Enable Scrolling Capture).
  - Chrome on Mac: "Allow JavaScript from Apple Events" (Chrome 68+).
  - The Mac App Store build shows no arrows.
- **Unsupported or problem cases:**
  - PDFs in reader apps (Win; use Snagit Printer).
  - Pages with several nested scroll areas.
  - Excel chart sheets, custom views, Print Preview and Page Layout; Excel auto-scroll only scrolls the upper-left pane.
  - Parallax sites; non-100% zoom; browser extensions.
  - Security software (Kaspersky, Trusteer Rapport).
  - Running elevated may help (Win).
- **Apps with stated support or fixes:** Chrome, Edge, Firefox, Safari, Brave, Microsoft Word (Mac 2023.2.0), Xcode, Finder folders, Mail, Power BI. Long Safari and Chrome captures got accuracy fixes. Long captures are more accurate (Mac 2026.0.1); Edge fixed (Mac 2026.3.2).
- **Version notes:** The user can keep working in other apps while processing runs (Win 2023.2.0).
- **COM equivalent:** Direction and starting position (current, top, left, top-left).
- **Platform:** Both. **Sources:** [T-SCR][T-IMG][T-HK][KB-SCRMAC][KB-SCRFAIL][KB-NOARROW][KB-EXCEL][KB-SCRINC][EXT-MACWORLD][VH-W23][VH-M23][VH-M24][VH-M26][COM25]

---

## 7. Grab Text (OCR at capture time)

- **Name:** Grab Text.
- **Steps:**
  1. Image tab > Selection = **Grab Text**. Incompatible options are disabled.
  2. Use the global hotkey or the Capture button (Mac doc lists Shift+Ctrl+C, the same keys as Control+Shift+C).
  3. Drag over text.
  4. A **Grab Text Results** dialog shows the recognized text with **Copy All**.
- **Results dialog options:**
  - A plain-text toggle ("Plain text" on Win, "Make Plain Text" on Mac) removes font and colour formatting (in capture since Win 2023.0.2).
  - Paragraph spacing is kept (Win 2026.4.0).
  - Esc closes the dialog (Win 2022.0.2).
  - Minimize button removed (Win 2024.3.6).
  - Spell check of the output is an Editor preference (Win).
- **Without opening the Editor:** Supported on Win 2023.0.0 and Mac 2022.0.2. Mac shows success or failure feedback when no window appears (2022.0.3).
- **Related Editor features (other team):** Grab Text on an existing image or selection (right-click), Edit Text, and Grab Text from a paused video frame (Mac 2022.2.0; Mac shortcut Shift+Command+O).
- **OCR engine and languages:**
  - Historically the ABBYY engine.
  - **Microsoft OCR is the default** from Win 2026.4.0.
  - An experimental "Optimized OCR" option exists (2026; Editor Preferences > Experiments).
  - Languages: English, French, German, Spanish and Portuguese (Spanish and Portuguese added 2021.0.1). Japanese support is stated inconsistently (?).
- **Recognition tips from support:**
  - Use high contrast and readable size.
  - Straighten rotated text.
  - Crop out bounding boxes.
  - Avoid huge scrolling captures, which are slow.
  - Decorative, handwritten, outlined or shadowed fonts may fail.
  - The first OCR run in a session is slower.
  - Fonts are mapped to Arial, Times New Roman or Courier New by family.
- **Version notes:** Better results on tall captures (Mac 2026.1.0). A "Cannot recognize text" error exists. OCR licence-file fixes in 2026.1.1, 2025.4.2 and 2024.3.7 kept the feature working.
- **Platform:** Both. **Ver:** Grab Text has existed since at least Snagit 2018 (?). **Sources:** [T-COMMON][T-TXT][T-IMG][KB-OCRBP][KB-OCR26][VH-W26][VH-W24][VH-W23][VH-W22][VH-W21][VH-M22][VH-M26][T-HK][RN-W26]

---

## 8. Step Capture (automatic click-through guides)

### 8.1 Starting and controlling a Step Capture
- **Starting:** Capture Window > **Presets > Step Capture**, or Image tab > Selection = **Steps**.
  - A "capturing steps" banner appears; wait for it to clear before clicking.
  - On Mac, clicking the banner dismisses it (2025.1.0).
- **Toolbar** (starts at the lower-left, draggable):
  - Live **step counter**.
  - **Pause** and **Record** (resume).
  - **Stop**, which opens the guide in the Editor.
  - Clicks on the toolbar are ignored, and the toolbar never appears in screenshots.
  - **Esc** cancels the capture (Win 2025.1.0).
  - Can be paused from the tray icon (Win).
- **What creates a step:**
  - Each mouse click or drag produces a screenshot step.
  - Typing and keyboard shortcuts are **not** recorded; the user adds them in the Editor.
  - Right-clicks are detected and labelled as such (Win 2025.1.0).
  - Tiny cursor jitters no longer split into many steps (fix Win 2025.3.1).
- **Smart behaviour (AI or heuristic):**
  - Each step gets auto-generated text describing what was clicked (Win 2025.1.0; Mac not stated (?)).
  - The screenshot is zoomed and centred on the detected area, with the cursor and a click highlight as editable objects.
  - If the detected area is smaller than 200×200 px, Snagit takes a full-screen capture instead (Win 2025.4.0).
  - Can capture elevated or admin apps such as Task Manager (Win 2025.1.0).
- **Settings respected:**
  - Capture Cursor (fix Win 2025.3.0).
  - Share destination, including File and multiple destinations (fixes Win 2025.2.0 and 2025.3.0).
  - Effects such as Scale Image for 4K screens (Win).
- **Limits:**
  - No hard maximum on steps.
  - Performance may drop beyond about 50 steps (Win) or 100 (Mac).
  - Memory fix in Win 2025.4.0.
  - Crash recovery keeps an in-progress Step Capture (Mac 2026.3.0 / 2026.4.0).
- **Output (Editor and sharing teams):**
  - A guide template with a header, numbered image sections, text sections and a footer.
  - Exports: Word, PowerPoint/Keynote, Markdown, HTML, per-image files, Camtasia Online/Editor (2025.3.0–2026.4.0).
  - A Library category for Step Captures (2026.3.0).
- **Platform:** Both. **Ver:** **2025.0.0** (Feb 2025). **Sources:** [T-STEP][T-STEPJ][T-S2CO][VH-W25][VH-M25][VH-W26][VH-M26][RN-W25][RN-M25][EXT-REWORKED]

---

## 9. Presets

### 9.1 Capture presets
- **What they are:** Saved bundles of capture settings, shown under **Presets** in the Capture Window.
- **Running a preset:** Hover to reveal a red capture button and click it, or press the preset's own hotkey.
- **Creating a preset:**
  - Add Preset > "New preset from current settings", or "New Preset" to configure from scratch.
  - Name it with Enter.
  - Optionally add a hotkey (Win "Add Hotkey", Mac "+ Shortcut").
- **Editing:** Click the Properties icon next to a preset to edit, export and so on.
- **Reordering:** Drag the move handle (Mac 2025.4.0).
- **What a preset can store:**
  - Capture type (Image or Video tab)
  - Selection, including Fixed Region and Scrolling
  - Effects
  - Share and File settings
  - Preview in Editor
  - Copy to Clipboard
  - Cursor
  - Time Delay
  - Webcam and microphone for video
- **Import and export (Win):** File > Import Presets accepts `.snagpresets` files (multi-select); Properties > Export saves them.
- **Built-in presets:**
  - Scrolling Capture (Win)
  - Step Capture (2025+)
  - Open in Camtasia Editor video preset (2026.2.0 Win / 2026.3.0 Mac)
- **Version notes:**
  - Presets keep their shortcuts (Mac fix 2024.0.1 / 2023.2.4).
  - Mac Capture Window once showed only 3 presets (fixed 2022.2.0).
  - Preset icons refreshed (Mac 2026.0.1).
  - Tooltip theme fix (Win 2026.4.0).
- **Platform:** Both. **Sources:** [T-PRE][T-SPEED][T-COMMON][KB-MODES][VH-M25][VH-W26][VH-M26][VH-M24][VH-M23][VH-M22]

---

## 10. Global and in-capture hotkeys

**Defaults** (Win from the Hotkeys guide and Win printable guide; Mac from the Hotkeys guide, Mac printable guide and 2019 help):

| Action | Windows default | Mac default |
|---|---|---|
| Global capture (current Capture Window settings) | Print Screen | Control+Shift+C |
| All-in-One capture | — | Control+Shift+O |
| Image capture | — | Control+Shift+S |
| Video capture | — | Control+Shift+V |
| Mission Control capture | — | Control+Shift+M |
| Window (Exposé) capture | — | Control+Shift+W |
| Menu capture | — | Control+Shift+E |
| Repeat last capture | none (assignable) | Control+Shift+R |
| Show/Hide or exit Snagit Capture | Ctrl+Shift+X | Command+Q quits Snagit |
| Start / pause / resume recording | Shift+F9 | Control+Shift+Space |
| Stop recording | Shift+F10 | Control+Shift+V |
| Swap PiP and fullscreen camera | — | Control+Shift+F |
| Screen Draw on/off (video) | Ctrl+Shift+D | Control+Shift+D |
| Video from Images | Ctrl+Alt+V | Option+V |
| Grab Text (Editor) | — | Shift+Command+O |
| All-in-One toolbar: Image / Video / Scrolling | — | I / V / S |
| Step Capture cancel | Esc (2025.1.0) | Esc (?) |

- **During capture:** see the table in 3.4. Screen Draw tool keys (video) are A = arrow, S = square, 1 = step / restart numbering.
- **Customizing:**
  - Win: File > Capture Preferences > Hotkeys covers Global Capture, Show/Hide, Repeat Last Capture, video start/pause and stop, plus the "Allow Snagit to override other hotkey assignments" toggle.
  - Mac: Settings > Keyboard. Click a field, press keys; an undo arrow reverts and an X clears.
  - Mac allows custom shortcuts for share destinations (2019.1.2) and Share Link (2023.1.0).
- **Older Mac defaults (?):** 2021 help listed Command+Shift+Space for start/pause and Option+Shift+V for stop. These are older defaults.
- **Notes:** On Mac, Control+Shift+V both starts video capture and stops recording, depending on context. TechSmith notes the shortcuts were tested on a QWERTY layout.
- **Version notes:** Single-key hotkeys in Mac All-in-One fixed in 2021.4.5. Hotkeys occasionally stopping was fixed in Win 2026.3.0 and a hotkey crash in 2026.4.0.
- **Sources:** [T-HK][PDF-SCW][PDF-SCM][PDF-H19][PDF-H21][T-PREFW][T-PREFM][KB-PRTSC][VH-M19][VH-M23][VH-M21][VH-W26]

---

## 11. Capture-related preferences

### 11.1 Windows (File > Capture Preferences)
- **General tab:**
  - Run Snagit when Windows starts.
  - Interface: Capture Window, Capture Widget, or both.
  - Colour theme: Light, Dark or System.
  - Update channel Standard or Delayed (2026.4.0).
  - Auto-check for updates.
- **Capture tab, image-relevant items:**
  - "Hide Snagit during capture and remove Snagit controls from completed captures" (fixes 2025.3.0; faster hiding 2026.2.0).
  - "Open screenshots copied from system tools".
  - "Show magnifier".
  - "Simplified crosshairs".
  - "Require Start button for scrolling capture".
  - "Support HDR color settings".
  - Printer Capture Settings.
- **Capture tab, video items (out of scope):** muted-mic alert, auto-start recording (off by default), countdown, Ready to Record window, minimized controls, scale to 1080p, webcam, hardware acceleration, quality.
- **Hotkeys tab:** see section 10.
- **Editor Preferences items that affect capture:**
  - Library: auto-save new image captures (on by default; if off, unsaved captures are deleted when Snagit closes). Default library folder is Documents\Snagit, saved as .snagx.
  - Share: "Do not include capture source in shared file names and titles". By default names use the page title or source; when on, the date-time or a custom name is used.
  - Advanced: keep Editor running in the background.
  - Experiments tab: HDR Capture Fidelity, Quick Image Annotation, Optimized OCR.
- **Sources:** [T-PREFW][PDF-H19][VH-W26][VH-W25][VH-W24]

### 11.2 Mac (Snagit > Settings)
- **General tab:** keep Snagit in the menu bar (launch at login); Appearance (Capture Window / Capture Widget); update channel and automatic updates; reset "Don't Ask Me" warnings.
- **Capture tab:**
  - "Hide Snagit and controls when capturing".
  - Open system screenshots/videos in Snagit, with Keep original file.
  - Open AirDrop media.
  - "Require Start button for scrolling capture".
  - **Fix Capture Permissions** button (Screen Recording and Accessibility).
  - Video items (out of scope): frame rate, H.264 or H.265, downsample Retina, countdown, hide desktop icons, auto-start, Ready to Record, combine audio, system audio.
- **Keyboard tab:** see section 10.
- **Share tab:** destination list, multiple named instances, "include capture source in file names".
- **Advanced tab:**
  - Scale down Retina images when sharing (on by default; hold Shift while dragging to bypass).
  - Keep a duplicate editable .snagx when saving to PNG and similar.
  - Default sharing format (JPG or PNG; used by the File share).
  - Library location (default ~/Pictures/Snagit).
  - Black-and-white menu bar icon.
- **Experiments tab:**
  - Single-window recording.
  - Fullscreen-as-layers.
  - QR code create, share and scan-during-capture (2025.4.0).
  - Screen Draw anytime (2026.3.0).
- **Sources:** [T-PREFM][VH-M25][VH-M26][KB-SCRMAC][PDF-H19]

### 11.3 Capture sounds and notifications (?)
- **Sounds:** No current public document describes a capture-sound or shutter-sound option on either platform.
- **Notifications that are documented:**
  - Mac shows a "Capture Copied" notification; clicking it brings the Editor forward (2019.1.1).
  - Grab Text shows success or failure feedback (Mac 2022.0.3).
  - Delay and video countdowns.
- **Platform:** Unknown. **Sources:** [VH-M19][VH-M22][T-PREFW][T-PREFM]

---

## 12. What happens after a capture (capture-side responsibilities)

### 12.1 Where the capture goes
- **Default routing:**
  - Opens in the Editor if Preview is on.
  - Copies to the clipboard if that is on.
  - Sends to the Share destination(s).
  - Auto-saves to the Library as .snagx unless disabled.
- **Notes:**
  - Captures appear in the Recent Captures tray (1–200 items).
  - On Mac, Retina captures are flagged; panoramic and scrolling captures too (2021.4.0).
  - A Library-location prompt on first capture was removed (Mac 2025.2.1).
  - Image captures had wrong resolution in some cases (fixed Win 2025.0.0).
- **Platform:** Both. **Sources:** [T-PREFW][T-PREFM][T-IMG][VH-M25][VH-M21][VH-W25]

### 12.2 Capture metadata and the Capture Info effect
- **What is recorded at capture time:**
  - Capture date and time
  - Application name and version
  - Website URL for browsers
  - Operating system (fixes Win 2026.2.0 and 2025.4.1)
  - Build information
- **Uses:**
  - The **Capture Info** effect overlays this metadata (font, size, colour and background options). Mac can prompt for custom text at capture (2023.2.0).
  - The Library groups captures by application or website (video metadata since 2023.2.0).
- **Browser URLs:** On Windows the URL replaces the browser name. Chrome URLs need Chrome's accessibility modes turned on (chrome://accessibility).
- **Limits:** The effect only works on captures made with Snagit.
- **Platform:** Both. **Sources:** [T-CI][T-FX][KB-CIURL][VH-W26][VH-W25][VH-M23][VH-W23][VH-W22][VH-M24]

### 12.3 Quick Image Annotation (experimental)
- **What it does:** One flow that captures, lets the user mark up in place, and copies to the clipboard without opening the Editor. Turned on in Editor Preferences > Experiments.
- **Platform:** Win. **Ver:** 2026.1.0. **Sources:** [VH-W26]

### 12.4 QR code scanning during capture (experimental)
- **What it does:** Reads QR codes while capturing; companion features create QR codes and give shared captures a code.
- **Platform:** Mac. **Ver:** 2025.4.0. **Sources:** [VH-M25]

---

## 13. AI and "smart" features near capture (2024–2026)
- **Step Capture auto-text and right-click detection** (Win 2025.1.0): see section 8.
- **Smart Redact:** Detects and redacts sensitive data such as card numbers, addresses, phones and faces. AI-based, on-device. It is an **Editor** feature, but runs on captured images.
  - Introduced 2025.0.0, after an experiment in Mac 2024.2.3 that auto-blurred phones, emails and faces in captures.
  - Accuracy improved by Optimized OCR (2026.3.0).
- **Screentelligence:** TechSmith's marketing name for local machine-learning layout and editing suggestions in 2025 (Editor).
- **Policy:** TechSmith states all AI runs on-device. Core AI features cannot be turned off; experimental features can be disabled at MSI deployment.
- **Platform:** Both. **Sources:** [VH-W25][VH-M24][VH-W26][EXT-REWORKED][KB-AI][RN-M24]

---

## 14. Permissions, platform requirements and admin controls
- **macOS permissions:**
  - **Screen & System Audio Recording** must be granted to Snagit (and SnagitHelper on 2022 and older). Without it only the desktop is captured, and macOS updates can reset the grant.
  - **Accessibility** is needed for automatic scrolling.
  - Settings has a "Fix Capture Permissions" helper.
  - Snagit 2026 needs macOS 15 or later (Sonoma dropped); Snagit 2025 dropped Ventura.
- **Windows:**
  - The Print Screen ownership issues in 1.4.
  - Running elevated helps capture elevated windows. Step Capture handles elevated apps since 2025.1.0.
  - Can run in VM or remote sessions (toolbar fixes in 2025.2.0 and 2026.1.0).
- **Admin deployment:**
  - Disable video capture (Mac 2024.4.0; Win Enterprise Deployment Tool).
  - Control the update channel (2026.4.0).
  - Disable experiments.
- **Sources:** [KB-MACPERM][KB-SCRMAC][T-PREFM][VH-M26][VH-M25][VH-M24][VH-W26][VH-W25][KB-AI]

---

## 15. Version timeline (capture-relevant highlights)

| Version | Platform | Change |
|---|---|---|
| Snagit 10 (2010) | Win | All-in-One capture, transparency for rounded windows, better auto-scroll, magnifier |
| Snagit 12 | Win | Simplified crosshairs |
| Snagit 13 / Mac 4 (2016) | Both | Panoramic capture |
| 2018 | Both | Grab Text present (?) |
| 2019.1.0 | Win | Snipping Tool / Snip & Sketch hand-off |
| 2019.1.1 | Mac | Auto-import system screenshots; Keep original |
| 2019.1.2 | Win | Mixed-DPI monitors |
| 2020.0.0 | Win | Video from Images (video) |
| 2020.2.0 | Mac | Fixed Region |
| 2021.0.1 | Both | Spanish and Portuguese OCR |
| 2021.1.0 | Mac | Scale/rotate as preset effects |
| 2021.1.1 | Mac | Standard-size guides for images |
| 2021.3.0 | Both | Capture UI refresh; switch accidental video to image |
| 2022.0.2 (Mac) / 2023.0.0 (Win) | Both | Grab Text without the Editor |
| 2022.1.0 | Win | Cancelable time delay |
| 2023.2.0 | Both | Panoramic renamed Scrolling; old Scrolling renamed Automatic Scrolling; header/footer handling |
| 2024.1.0 | Win | Hide Snagit UI from scrolling and video results |
| 2024.2.2 | Win | HDR capture |
| 2024.3.0 | Mac | Menu bar right-click with All-in-One |
| 2025.0.0 | Both | Step Capture; Smart Redact |
| 2025.0.0 | Mac | Experiments: window layers, single-window recording |
| 2025.1.0 | Win | Step Capture auto-text, right-click labels, elevated apps, Esc to cancel |
| 2025.2.0 | Mac | 5x faster crosshairs |
| 2025.2.1 | Mac | Corner rounding at capture |
| 2025.4.0 | Mac | QR experiments; preset drag-reorder |
| 2025.4.0 | Win | HDR Capture Fidelity experiment; Step Capture full-screen fallback under 200×200 |
| 2026.0.0 | Mac | Window selection type; refreshed UI |
| 2026.1.0 | Mac | Schedule and Interval captures |
| 2026.1.0 | Win | Quick Image Annotation experiment |
| 2026.3.0 | Both | Step Capture to Camtasia; Optimized OCR experiment |
| 2026.3.1 | Win | Interval minimum 3 s |
| 2026.4.0 | Win | Microsoft OCR default; paragraph spacing; Clipboard capture fix |

**Sources:** all version-history keys in section 17, plus [EXT-CAMPUS][NEWS2016][KB-SLUG]

---

## 16. Open questions to resolve for parity (?)
1. The exact File-share settings dialog: naming tokens, counter, "ask for name", overwrite behaviour. Only the COM model is public.
2. Capture sound or shutter options. No public documentation found.
3. The current Windows Selection list beyond Region, Window, Fullscreen, Scrolling, Grab Text, Menu, Freehand, Multiple Area and Steps: whether Object, File and Clipboard remain in the dropdown, and how Clipboard text is rendered.
4. How Multiple Area captures are laid out in the final image.
5. Mac modifier keys during selection (square, aspect ratio, arrow-key nudge); public sources conflict.
6. Mac Capture Widget behaviour and position options.
7. Whether Step Capture auto-text exists on Mac, and what the Mac cancel key is.
8. Japanese OCR support, and which OCR engine is the Mac default.
9. The full list of effects available at capture time versus Editor-only.
10. Windows webcam still capture (appears absent).

---

## 17. Deduplicated list of URLs consulted, with keys

**TechSmith Learn tutorials and pages**
- [T-CAP] https://www.techsmith.com/learn/tutorials/snagit/how-to-capture-your-screen/
- [T-CAPJ] https://www.techsmith.com/learn/wp-json/wp/v2/tutorial/145406
- [T-IMG] https://www.techsmith.com/learn/tutorials/snagit/image-capture/
- [T-SCR] https://www.techsmith.com/learn/tutorials/snagit/scrolling-capture/
- [T-STEP] https://www.techsmith.com/learn/tutorials/snagit/step-capture/
- [T-STEPJ] https://www.techsmith.com/learn/wp-json/wp/v2/tutorial/278279
- [T-S2CO] https://www.techsmith.com/learn/tutorials/snagit/step-capture-to-camtasia-online/
- [T-COMMON] https://www.techsmith.com/learn/tutorials/snagit/common-captures/
- [T-PREFW] https://www.techsmith.com/learn/tutorials/snagit/preferences/
- [T-PREFM] https://www.techsmith.com/learn/tutorials/snagit/preferences-mac/
- [T-HK] https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/
- [T-GHK] https://www.techsmith.com/learn/tutorials/snagit/change-global-hotkey/
- [T-PRE] https://www.techsmith.com/learn/tutorials/snagit/presets/
- [T-TXT] https://techsmith.com/learn/tutorials/snagit/edit-image-text
- [T-FX] https://techsmith.com/learn/tutorials/snagit/effects
- [T-CI] https://www.techsmith.com/learn/tutorials/snagit/add-capture-information/
- [T-SHARE] https://www.techsmith.com/learn/tutorials/snagit/share/
- [T-CUST] https://www.techsmith.com/learn/tutorials/snagit/customize-snagit/
- [T-SPEED] https://techsmith.com/learn/tutorials/snagit/speed-up-snagit-workflow
- [T-PREV] https://www.techsmith.com/learn/tutorials/snagit/snagit-previous-version-help/
- [W-CW] https://www.techsmith.com/learn/webinars/learn-one-new-thing-the-snagit-capture-window/

**Version histories (support.techsmith.com)**
- [VH-WIDX] https://techsmith.com/snagit-version-history.html (redirects to https://support.techsmith.com/hc/en-us/sections/39094961933453)
- [VH-W26] https://support.techsmith.com/hc/en-us/articles/42674936732685-Snagit-Windows-2026-Version-History
- [VH-W25] https://support.techsmith.com/hc/en-us/articles/39102293598349-Snagit-Windows-2025-Version-History
- [VH-W24] https://support.techsmith.com/hc/en-us/articles/39102250364429-Snagit-Windows-2024-Version-History
- [VH-W23] https://support.techsmith.com/hc/en-us/articles/39102261751949-Snagit-Windows-2023-Version-History
- [VH-W22] https://support.techsmith.com/hc/en-us/articles/43410048034061-Snagit-Windows-2022-Version-History
- [VH-W21] https://support.techsmith.com/hc/en-us/articles/43410747342605-Snagit-Windows-2021-Version-History
- [VH-W20] https://support.techsmith.com/hc/en-us/articles/43410801744525-Snagit-Windows-2020-Version-History
- [VH-W19] https://support.techsmith.com/hc/en-us/articles/43414049971213-Snagit-Windows-2019-Version-History
- [VH-MIDX] https://support.techsmith.com/hc/en-us/articles/115006443147
- [VH-M26] https://support.techsmith.com/hc/en-us/articles/41975263481613-Snagit-Mac-2026-Version-History
- [VH-M25] https://support.techsmith.com/hc/en-us/articles/37938520706957-Snagit-2025-Version-History
- [VH-M24] https://support.techsmith.com/hc/en-us/articles/37938485483021-Snagit-Mac-2024-Version-History
- [VH-M23] https://support.techsmith.com/hc/en-us/articles/37938496753549-Snagit-Mac-2023-Version-History
- [VH-M22] https://support.techsmith.com/hc/en-us/articles/43415422714125-Snagit-Mac-2022-Version-History
- [VH-M21] https://support.techsmith.com/hc/en-us/articles/43415439595533-Snagit-Mac-2021-Version-History
- [VH-M20] https://support.techsmith.com/hc/en-us/articles/43415462314125-Snagit-Mac-2020-Version-History
- [VH-M19] https://support.techsmith.com/hc/en-us/articles/43415505223437-Snagit-Mac-2019-Version-History

**In-app release notes and changelog**
- [RN-W26] https://download.techsmith.com/update/snagit/enu/26_updatemessage.html
- [RN-W25] https://download.techsmith.com/update/snagit/enu/25_updatemessage.html
- [RN-M25] https://download.techsmith.com/update/snagitmac/enu/rnotes.html
- [RN-M24] https://download.techsmith.com/update/snagitmac/enu/2420/rnotes.html
- [WN-CMP] https://www.techsmith.com/snagit-compare-versions.html
- [WN-EXP] https://techsmith.com/snagit/whats-new/experimental-feature-improvements
- [WN-RCM] https://techsmith.com/snagit/whats-new/access-snagit-features-faster-with-the-new-right-click-menu

**Support knowledge-base articles and sections**
- [KB-SEC1] https://support.techsmith.com/hc/en-us/sections/200514918-Capture
- [KB-SEC2] https://support.techsmith.com/hc/en-us/sections/200514858-Capture?page=2 and https://support.techsmith.com/hc/en-us/sections/200514858-Capture-Video
- [KB-SEC3] https://support.techsmith.com/hc/en-us/sections/43315497268237-Capture-Image
- [KB-MODES] https://support.techsmith.com/hc/en-us/articles/360002743452-Capture-Modes-in-Snagit-Windows
- [KB-WIDGET] https://support.techsmith.com/hc/en-us/articles/360029429991-Pin-Snagit-to-the-Top-of-the-Screen-Using-Capture-Widget
- [KB-PRTSC] https://support.techsmith.com/hc/en-us/articles/203731428-Snagit-Print-Screen-Hotkey-Not-Working
- [KB-RLC] https://support.techsmith.com/hc/en-us/articles/203731758-Error-Unable-to-Register-Snagit-Repeat-Last-Capture-Hotkey
- [KB-MULTI] https://support.techsmith.com/hc/en-us/articles/203731188-Capturing-Only-One-Screen-with-a-Multiple-Monitor-Configuration
- [KB-SLUG] https://support.techsmith.com/hc/en-us/articles/203731288-Snagit-Freezes-Crashes-or-the-Crosshairs-Are-Sluggish-When-Trying-to-Take-a-Capture
- [KB-AUTODET] https://support.techsmith.com/hc/en-us/articles/40206365544333-Snagit-is-Not-Auto-Detecting-Windows-During-Capture
- [KB-MENU] https://support.techsmith.com/hc/en-us/articles/203731168-Capturing-a-Dropdown-Menu-in-Snagit-Windows
- [KB-FIXED] https://support.techsmith.com/hc/en-us/articles/360031128032-Capture-the-Same-Part-of-the-Screen-Using-Fixed-Capture-Video
- [KB-SCRPREV] https://support.techsmith.com/hc/en-us/articles/4407009878029-Scrolling-Preview-Window-and-Control-Bar-is-Captured-by-Snagit
- [KB-NOARROW] https://support.techsmith.com/hc/en-us/articles/206390367-Snagit-Windows-No-Scrolling-Arrows
- [KB-SCRINC] https://support.techsmith.com/hc/en-us/articles/203731338-Unable-to-Complete-Scrolling-Capture-in-Snagit
- [KB-EXCEL] https://support.techsmith.com/hc/en-us/articles/203731478-Microsoft-Excel-and-Snagit-Automatic-Scrolling-Capture-Problem
- [KB-SCRMAC] https://support.techsmith.com/hc/en-us/articles/203732578-Enable-Scrolling-Capture-on-macOS
- [KB-SCRFAIL] https://support.techsmith.com/hc/en-us/articles/219245257-Scrolling-Capture-Failed
- [KB-WEBMAC] https://support.techsmith.com/hc/en-us/articles/115002361531-Web-Capture-With-Snagit-on-macOS
- [KB-MACPERM] https://support.techsmith.com/hc/en-us/articles/360034561752-Only-Desktop-Captured-Using-Snagit-on-Mac
- [KB-OCRBP] https://support.techsmith.com/hc/en-us/articles/115002031592-Snagit-Grab-Text-Edit-Text-Best-Practices
- [KB-OCR26] https://support.techsmith.com/hc/en-us/articles/45409168722445-Issues-With-Grab-Text-or-Other-OCR-Functions-Such-as-Edit-Text-or-Smart-Redact
- [KB-HDR] https://support.techsmith.com/hc/en-us/articles/25817515057549-Snagit-Capture-is-Washed-Out-With-HDR
- [KB-ROUND] https://support.techsmith.com/hc/en-us/articles/4410622169869-Capturing-Rounded-Corners-in-Windows-11
- [KB-CIURL] https://support.techsmith.com/hc/en-us/articles/360018564392-Snagit-Capture-Info-Show-URL-in-Chrome
- [KB-SAVEFMT] https://support.techsmith.com/hc/en-us/articles/203731148-Automatically-Save-Images-to-a-Specific-Format-During-Capture
- [KB-FMT] https://support.techsmith.com/hc/en-us/articles/360030581892-Automatically-Capture-to-a-Specific-File-Format-in-Snagit-Video
- [KB-AI] https://support.techsmith.com/hc/en-us/articles/46075165106957-How-can-I-disable-AI-features-in-Snagit-and-Camtasia

**Official help PDFs, shortcut guides and COM documentation (public documentation only; no binaries)**
- [PDF-H24] https://www.techsmith.com/learn/wp-content/uploads/2025/01/Snagit-Help-2024-EN.pdf
- [PDF-H23] https://www.techsmith.com/learn/wp-content/uploads/2023/09/Snagit-Help-2023-EN.pdf
- [PDF-H22] https://www.techsmith.com/learn/wp-content/uploads/2022/10/Snagit-Help-2022.pdf
- [PDF-H21] https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-Help-2021.4.pdf
- [PDF-H20] https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit2020.1.3-Help.pdf
- [PDF-H19] https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-EN-Help-2019.1.1.pdf
- [PDF-W18] https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-Windows-Help-2018.1.pdf
- [PDF-M18] https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-Mac-Help-2018.1.pdf
- [PDF-13] https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-13.pdf
- [PDF-SCM] https://www.techsmith.com/learn/wp-content/uploads/2024/07/Snagit-Shortcuts-Guide-Printable-Mac.pdf
- [PDF-SCW] https://www.techsmith.com/blog/wp-content/uploads/2024/02/Snagit-Shortcuts-for-Windows.pdf
- [COM12] https://assets.techsmith.com/Docs/pdf-snagit/COM_Documentation.pdf
- [COM25] https://assets.techsmith.com/Docs/Snagit-2025-COM-Server-Guide.pdf
- [NEWS2016] https://assets.techsmith.com/Downloads/pdf-landingpages/July_News2016.pdf

**TechSmith blog**
- [BLOG-MAC] https://www.techsmith.com/blog/screen-capture-mac
- [BLOG-HIDDEN] https://www.techsmith.com/blog/snagit-hidden-features/
- [BLOG-253406] https://techsmith.com/blog/?p=253406

**Third-party press and reviews**
- [EXT-NEO26] https://www.neowin.net/amp/snagit-20260/
- [EXT-NEO2630] https://www.neowin.net/amp/snagit-202630/
- [EXT-NEO2540] https://www.neowin.net/amp/snagit-202540/
- [EXT-REWORKED] https://www.reworked.co/the-wire/snagit-and-camtasia-2025-introduce-ai-and-screentelligence-powered-workflows-for-faster-more-impactful-content-creation/
- [EXT-WIKI] https://en.wikipedia.org/wiki/Snagit
- [EXT-MACWORLD] https://www.macworld.com/article/220563/review-snagit-should-cover-all-your-mac-screenshot-needs.html
- [EXT-CAMPUS] https://campustechnology.com/Articles/2010/05/12/Snagit-10-Screen-Capture-Tool-Gets-Transparency-Support-All-in-One-Capture.aspx?p=1
- [EXT-INDEZINE] https://www.indezine.com/products/other/snagit-for-mac.html