# Snagit Windows and Mac: feature specification (public sources only, as of 12 Oct 2026)

**How I worked:** I used only TechSmith's public pages: version histories, tutorials, KB articles, system requirements and the changelog. I also used a few press and aggregator pages. I did not download, install or inspect any binary, installer or .snagx file. Everything below is paraphrased.

**Key facts first:**
- **Versions are numbered separately per platform.** The numbers do not line up between Windows and Mac. For example, Mac 2026.2.0 only changed licensing, while Windows 2026.2.0 added sharing videos to Camtasia Online. Release dates also differ: Mac 2026.0.0 shipped 16 Dec 2025, Windows 2026.0.0 on 14 Jan 2026.
- **Latest releases:** Windows **2026.4.0** (6 Oct 2026) and Mac **2026.4.1** (9 Oct 2026). Security-only maintenance continued on older lines: Win 2025.4.3 and 2024.3.8 (both 11 Jun 2026); Mac 2025.4.1 (30 Jul 2026) and 2024.4.10 (28 Jul 2026).
- **Release model change in 2026:** The Windows 2026.0.0 notes say the product is now just "Snagit", with no year in the name and no yearly reinstall. Branding changed from "TechSmith Snagit" to "Camtasia Snagit". The Feb 2025 press release said 2025 would be the last annual release before moving to continuous delivery.

**Source keys** (full URLs are in the list at the end):
- Version histories: W24, W25, W26 (Windows) and M24, M25, M26 (Mac).
- In-app "What's New" pages: WN25, WN26. Mac 2024 release notes: MRN24. Public changelog: CHLOG.
- AI disclosure page: AI3P.
- Tutorials and KB articles: named by topic (e.g., SHARE, HK, STEPOUT, SYSREQ-WEB).

---

## 1. Version timeline (new features and behavior changes only)

### 1A. Snagit for Windows

**2024 line** (source: W24)
- **2024.0.0 (3 Oct 2023)**
  - In-product tours and a redesigned Learning Center.
  - Combine Videos window and a new Video menu.
  - Pin captures in the Recent Captures Tray.
  - Click Animation in recordings.
  - Microsoft Teams share destination; OneDrive Personal share destination.
  - Slack sharing can target channels or DMs.
  - New "Starter" theme and Quick Styles. Screen Draw colors and arrowheads updated to match.
  - Stamps refreshed; stamp shadow now off by default.
  - No longer prompts for a Library location.
  - Unsaved cuts are applied automatically when combining videos.
  - Recording toolbar splits toggles from their dropdowns.
  - Better sync between screen, mic and system audio.
- **2024.0.1 (17 Oct 2023):** Reset-to-defaults button (Editor Prefs > Advanced). Faster click animation. Tray keeps captures when the Library is moved.
- **2024.0.2 (29 Nov 2023):** UI language written into PDF metadata. Screen-reader improvements on the Editor toolbar.
- **2024.0.3 (19 Dec 2023)**
  - Tray holds up to 200 files.
  - Videos can be tagged in the tray.
  - Links to more stamps on TechSmith Assets.
  - New updater UI.
  - Enterprise Deployment Tool can skip installing SQL CE.
  - The "Record a Video" tour is hidden when video is disabled.
- **2024.1.0 (6 Mar 2024)**
  - Muted-mic alert during recording.
  - New stamps.
  - One-click switch between fullscreen webcam and picture-in-picture.
  - On Win10 20H1+, Snagit's own toolbars can be excluded from scrolling and video captures.
  - Library shows PNG/JPG files saved in the Library folder.
  - Feedback button in the Editor.
  - Option to keep capture-source info out of shared file names.
  - Dark mode extended to sign-in dialogs.
  - Admin: option to disable the welcome window; Library conversion tool for installs without SQL CE.
- **2024.1.1 (12 Mar 2024):** Subscription purchase option.
- **2024.1.2 (25 Apr 2024)**
  - "Ready to Record" window to review settings before recording.
  - Default file type when dragging out of the tray or Library.
  - Google Drive sharing goes only to a TechSmith folder.
  - Common keyboard shortcuts added.
  - All-in-One captures no longer rounded to even pixel sizes.
  - Admin: exclude SQL CE; `TSC_UPDATER_PROXYLIST_VALUE` sets proxies for the updater.
- **2024.1.4 (6 Jun 2024)**
  - Video files 10–20% smaller.
  - Modern folder picker.
  - Screencast Pro has unlimited hosting.
  - Admin: MSI uninstall no longer needs admin rights.
- **2024.2.0 (6 Aug 2024)**
  - New recording engine supporting 60 fps.
  - Tray inside Combine Videos.
  - Save a timeline selection as a new video.
  - Share destinations can be toggled without restarting.
  - Account dropdown shows profile photo and an update check.
  - Message when email sharing fails with the new Outlook on Win11.
  - Silent EXE install exits with a success code.
- **2024.2.2 (21 Aug 2024)**
  - HDR image capture (setting in Capture Preferences). Not supported for scrolling or video captures.
  - Editor button to send a video to Camtasia.
  - License info shown in the Account dropdown.
- **2024.3.0 (13 Nov 2024)**
  - Rounded image corners (corner handle; one click gives a 20 px radius).
  - Assets button opens TechSmith Assets, including AI-generated images.
  - Notice about usage-data collection.
  - Explorer shell extension no longer installed by default.
  - Installer asks the user to acknowledge the privacy policy.
  - EULA now covers YouTube API use.
- **2024.3.1 (15 Jan 2025):** Knowmia share destination removed (service retired).
- **2024.3.2 (5 Feb 2025):** Better white-level accuracy on HDR monitors. Better licensing error handling.
- **2024.3.3 (5 Aug 2025):** Panopto share destination removed. Capture Widget reliability fixes.
- **2024.3.5 (13 Nov 2025):** Grab Text can copy plain text. Arrow keys step numeric fields. Slack authorization method changed.
- **2024.3.6 (19 Mar 2026):** Grab Text Results dialog loses its minimize button.
- **2024.3.7 (1 Apr 2026):** OCR licensing file updated so Grab Text, Edit Text and Smart Redact keep working. The same fix shipped in 2025.4.2 and 2026.1.1.

**2025 line** (sources: W25, WN25, PR25)
- **2025.0.0 (12 Feb 2025)**
  - **Step Capture**: builds a numbered, editable guide from your clicks.
  - **Smart Redact**: detects sensitive data and redacts it.
  - **Remove Background Noise** for recordings.
  - Admins can point Share Link to OneDrive, Google Drive or Screencast.
  - Annual subscriptions for individuals and businesses; business seats can be transferred.
  - Share-complete notices use Windows notifications.
  - Template changes: shapes can be drawn over drop zones, zoom steps of 20%, a delete-section icon, text labels in Add Section.
  - Knowmia removed.
  - Installer is now 64-bit (WiX 4).
- **2025.1.0 (26 Mar 2025)**
  - Step Capture auto-writes descriptive text for each clicked item. Right-clicks are labeled as such. Elevated apps such as Task Manager can be captured. Esc cancels.
  - Smart Redact: solid redaction color can be changed; corners are rounded; no duplicate categories on one item; fewer false face detections (side and 3/4 profiles now handled).
  - Blur Quick Styles apply only type and intensity.
  - All license types supported.
  - HDR white accuracy improved.
  - Warning when a Screencast description is too long.
- **2025.1.1 (17 Apr 2025):** Notice when a license is disabled.
- **2025.2.0 (5 Jun 2025)**
  - **Edit in Snagit from Camtasia** (needs Camtasia 2025.2+): right-click media in the Camtasia bin, edit in Snagit, then send back.
  - Step Capture template fits longer text; callouts and arrows stay anchored when sections move.
  - Faster Capture app startup.
- **2025.2.1 (25 Jun 2025):** In-app update no longer relaunches Snagit with admin rights.
- **2025.3.0 (6 Aug 2025)**
  - Step Capture cursors can be moved or deleted.
  - Step Capture shares to Word as separate images plus editable text.
  - Main toolbar shows both Library and Editor.
  - Offline help PDF fallback.
- **2025.3.1 (3 Sep 2025):** Deployment Tool can remove Assets service links (for offline or restricted sites). EULA §7.1 updated. Fix for an untrusted-search-path privilege-elevation issue.
- **2025.4.0 (23 Oct 2025)**
  - **Share images to Camtasia Online.**
  - Experiment: **HDR Capture Fidelity**.
  - Step Capture falls back to a full-screen capture when the detected area is under 200×200 px.
  - Multiple files can be sent to Camtasia Editor at once.
  - Right-clicking the title bar opens the system menu.
  - Share destinations disabled at deployment stay disabled across updates.
- **2025.4.1 (23 Mar 2026):** Installer moves to WiX 6.

**2026 line (continuous delivery)** (sources: W26, WN26, NEO2600, NEO2630)
- **2026.0.0 (14 Jan 2026)**
  - App renamed to plain "Snagit"; subscribers no longer reinstall each year.
  - Branding becomes "Camtasia Snagit".
  - Experiment: improved OCR for finding and editing text.
  - Subscription-expired dialog offers sign-in or account switch.
  - Guidance for Google Drive and YouTube permissions.
  - Spell-check dictionary includes TechSmith product names.
  - Sliders adjustable with arrow keys.
- **2026.1.0 (13 Mar 2026)**
  - **Record to Camtasia Editor**: a new "Open In" setting replaces "Preview in Editor". Choices are Snagit Editor, None (save to Library only) or Camtasia Editor (multitrack).
  - Experiment: **Quick Image Annotation**.
  - Step Capture to PowerPoint creates a slideshow: title slide, one slide per image, editable text.
  - **Select All** like objects from Tool Properties.
  - Ready to Record window shows the target editor.
  - Plugin version numbers removed from Share Preferences.
- **2026.1.1 (1 Apr 2026):** OCR licensing file updated (same as 2024.3.7 above).
- **2026.2.0 (30 Apr 2026)**
  - **Share videos to Camtasia Online**: multiple videos can be combined into one project, with collaborators.
  - Capture preset that opens recordings in Camtasia Editor.
  - Email sharing works with classic and new Outlook.
  - "System" color theme.
  - Dialogs styled like native Windows dialogs.
  - Offline message for expired subscriptions.
  - Fill tool works on floating images. **Behavior change:** clicking an annotation with Fill no longer fills the canvas.
  - Snagit hides faster at capture time.
- **2026.2.2 (28 May 2026):** Snagit can be activated through a Camtasia Editor subscription that includes it.
- **2026.3.0 (16 Jul 2026)**
  - **Step Capture to Camtasia Online** and **to Camtasia Editor** (Camtasia Editor 2026.2+; steps land on the timeline).
  - Experiment: Optimized OCR improves Smart Redact accuracy.
  - Step Capture image-count badges and a Library category.
  - Refreshed colors and icons.
  - Import Backup and Share > Email return.
  - OneDrive, SharePoint and Teams sign-in uses built-in Windows authentication.
- **2026.3.1 (23 Jul 2026)**
  - .snagx (with cursor and text data) is sent to Camtasia Online.
  - **Minimum Interval capture raised from 1 s to 3 s.**
  - Email compose options restored.
  - OpenSSL 3.6.3; untrusted-search-path fix.
- **2026.4.0 (6 Oct 2026)**
  - **Record directly to Camtasia Online** (Video tab > Open In), with teammate collaboration.
  - **Update channel**: Standard (immediate) or Delayed (after an extended rollout). Admins can set it and lock it.
  - **Microsoft OCR becomes the default OCR engine.**
  - Step Captures sent to Camtasia Online include image text.
  - "?" help menu in the Editor.
  - **X (Twitter) share destination removed.**
  - Email destination handles wide images better.
  - Status text reads "Sending…" for local destinations and "Uploading…" for cloud ones.
  - Grab Text keeps paragraph spacing.

### 1B. Snagit for Mac

**2024 line** (sources: M24, MRN24, WN24)
- **2024.0.0 (3 Oct 2023)**
  - Combine Videos window.
  - Click Animation with custom colors.
  - **Blurred webcam background.**
  - One-click or hotkey switch between fullscreen webcam and PiP.
  - New Learning Center.
  - Starter theme and Quick Styles.
  - Stamps refresh.
  - Teams sharing.
  - Better Share Link notifications.
  - New Video menu (Jump To moved there).
  - Capture Info can include the system username.
  - Rename files from the File menu.
- **2024.1.0 (14 Dec 2023)**
  - 60 fps ("Very High") recording.
  - Save a timeline selection as a new video.
  - Capture resumes automatically after permissions are granted.
  - Experiments: smart text selection for Highlight/Blur/Spotlight; Oval and Pen Screen Draw tools; corner rounding for canvas and floating images; pixel distance measurement.
- **2024.2.0 (28 Feb 2024)**
  - Muted-mic alert.
  - Stamps refresh.
  - Experiment: **Capture Hotkeys in Video** (shows pressed key combinations in the recording).
  - No processing delay when sharing edited videos.
  - Admin: **the Experiments tab is hidden when analytics are disabled at deployment.**
- **2024.2.1 (12 Mar 2024):** Subscription purchase. Dragging videos from the tray keeps edits.
- **2024.2.3 (2 Apr 2024)**
  - Ready to Record window.
  - Videos are optimized before Screencast upload.
  - Experiment: auto-detect and blur sensitive data (phone numbers, emails, SSNs, card numbers, faces). This later became Smart Redact.
  - Dropbox: rename/overwrite and cancel.
  - New emoji stamps.
- **2024.2.5 (22 May 2024)**
  - **Last release for macOS 12.**
  - Tray inside Combine Videos.
  - Google Drive uploads go to a TechSmith folder only.
  - Line Spacing for Text and Callout.
  - Warning if .snagx autosave fails.
  - Live Measure and Selection Auto-Fill about 3× faster.
- **2024.3.0 (6 Aug 2024)**
  - **Background Noise Removal.**
  - **AirDrop to Snagit** (received files open automatically).
  - **Hide Desktop Icons** while recording.
  - Right-click Quick Menu on the menu-bar icon.
  - **Behavior change:** JPEG files now save as .jpg instead of .jpeg.
  - Screencast optimization can be cancelled.
- **2024.3.2 (12 Sep 2024)**
  - Assets button (stamps, templates, themes, photos, AI images).
  - **Make a Silent Copy** of a video.
  - Unflatten objects.
  - Account dropdown works even when sign-in is disabled.
  - Google Workspace restricted-access sharing.
  - Better Safari auto-scrolling.
- **2024.4.0 (14 Nov 2024)**
  - **Rounded image corners.**
  - **Add Narration** over images (makes a video or slideshow).
  - Context tips.
  - **Smart Move changed from a toggle to a button.**
  - Send-to-Camtasia button.
  - Square eraser.
  - Admin option to deploy with video capture disabled.
- **2024.4.1 (27 Feb 2025):** Word sharing inserts at the cursor. Knowmia removed.
- **2024.4.2 (25 Mar 2025):** Slack authorization stored in the TechSmith Account and synced across devices.
- **2024.4.5–4.10:** Stability work for macOS 26 Tahoe.
  - 2024.4.9: Dropbox component replaced ahead of its 2026 end of support.
  - 2024.4.10: links must use HTTPS.

**2025 line** (sources: M25, PR25)
- **2025.0.0 (12 Feb 2025)**
  - **Step Capture.**
  - **Smart Redact.**
  - **Background noise removal.**
  - Admins can point Share Link to Google Drive or Screencast.
  - **Webcam virtual backgrounds** (built-in image, uploaded image or blur).
  - **Gradients** for canvas, Shape, Callout and Fill.
  - Subscriptions.
  - Pen added to Screen Draw.
  - Cmd-drag resizes from the center.
  - Knowmia removed.
  - Experiments: record a single app window even as it moves; full-screen capture with each window as a separate editable layer.
  - **macOS 13 dropped.**
- **2025.1.0 (1 Apr 2025)**
  - Step Capture to Word (editable).
  - Tray scrolls.
  - Template placeholders get a Capture button.
  - File > Export works for video.
  - System captures taken during a recording are imported later.
  - Slack authorization synced via the TechSmith Account.
- **2025.2.0 (8 May 2025)**
  - Step Capture: cursors can be moved or deleted; guides share to **Apple Pages**.
  - Smart Redact: solid color option and rounded corners.
  - Cut Out Quick Styles can be added to Favorites.
  - **Continuity Camera import** (iPhone/iPad).
  - **H.265 recording.** Files may not open on Windows 10.
  - Installer about 50 MB smaller.
- **2025.2.1 (10 Jun 2025):** Corner rounding can be set at capture time (Image tab > Effects). No first-capture prompt for the Library location.
- **2025.3.0 (21 Aug 2025)**
  - **"Open In"** (Snagit Editor / None / Camtasia Editor) replaces Preview in Editor.
  - **Step Capture Markdown export.**
  - Select All like objects.
  - **Distinct right-click color** in click animation.
  - Import .mov without conversion.
  - Drag emoji from the Character Viewer onto the canvas.
  - More File > Export options.
  - Screencast videos are viewable immediately.
  - .snagx saves up to 2× faster.
- **2025.3.1 (15 Sep 2025):** Better address detection in Smart Redact. Undo/redo for corner rounding.
- **2025.4.0 (4 Nov 2025)**
  - **Share images to Camtasia Online.**
  - **In-app Assets browser** (stamps, templates, themes).
  - **Camera border** (solid or gradient).
  - Experiment: **QR codes** (create custom codes, get a code for a shared capture, scan codes during capture).
  - Presets reorder by drag and drop.
  - Library tags that differ only in case are merged.
  - Menu icons in Tahoe style.
  - "?" help menu in the Editor.

**2026 line** (source: M26)
- **2026.0.0 (16 Dec 2025)**
  - **Window Capture** (Selection > Window) for image and video. Video keeps following the window when it moves or resizes.
  - Rename to "Camtasia Snagit".
  - UI colors and icons refreshed; new installer.
  - Step Capture to **PowerPoint and Keynote** slideshows.
  - **Smart Redact**: click text to add or remove a redaction; right-click for Redact All Matches, Remove, Edit Text.
  - **Highlight, Blur and Spotlight can apply only to text.**
  - New color picker.
  - Dropbox component updated.
  - **Panopto removed.**
  - **macOS 14 dropped**; Snagit 2025 is the last version for it.
- **2026.0.1 (4 Feb 2026)**
  - Better accuracy on long auto-scroll captures.
  - Faster large Libraries.
  - Resize dialog keyboard navigation.
  - Support Tool can relocate the Library.
- **2026.1.0 (11 May 2026)**
  - **Share videos to Camtasia Online.**
  - **In-app photo Assets.**
  - Pin and reorder items in the tray.
  - Resize, rotate and set corner radius on several objects at once.
  - Click a video to play or pause.
  - **Scheduled and Interval (time-lapse) capture.**
  - Batch-export Step Capture images.
  - Annotations are auto-selected after drawing (can be turned off).
  - Better Grab Text on tall captures.
  - Crash recovery for recordings sent to Camtasia.
  - Admin: disable the feedback form and TechSmith Fuse mobile connections.
- **2026.2.0 (28 May 2026):** Activation through a Camtasia Editor subscription.
- **2026.3.0 (21 Jul 2026)**
  - **Step Capture to Camtasia Online and Camtasia Editor.**
  - Experiment: **Screen Draw available at any time** (not only while recording).
  - **Step Capture HTML export.**
  - Step Capture Library category; save images as separate files.
  - Green screen-share borders removed from Zoom recordings automatically.
  - Preset for opening recordings in Camtasia Editor.
  - Custom Google Drive permissions.
  - Box shown in the Share menu by default.
  - Smart Redact faster.
- **2026.3.2 (2 Sep 2026)**
  - .snagx sent to Camtasia Online.
  - Word insertion choice: new document, current document or ask.
  - Tray drag-reorder improvements.
  - Admin can hide the Edit in Camtasia button.
- **2026.3.3 (14 Sep 2026):** macOS 27 fixes.
- **2026.4.0 (7 Oct 2026)**
  - **Record to Camtasia Online.**
  - **Standard/Delayed update channel.**
  - **Auto-stop recording timer** (h:m:s, Timer button on the recording toolbar).
  - Word and Pages: new vs open document, default set in Share Settings.
  - Better PDF pagination for scrolling captures.
  - **New enterprise deployment tool.**
- **2026.4.1 (9 Oct 2026):** Library crash fix.

---

## 2. AI-powered and cloud-dependent features

**What TechSmith says about AI.** Its own AI disclosure (AI3P, updated 6 Oct 2026) says:
- Every listed Snagit model runs **on the device**.
- None of them is generative AI.
- None are used for training.
- On-device models work without internet.

The Feb 2025 press release calls this "Screentelligence" and says metadata is analyzed locally (PR25).

| Feature | What it does | Technology per TechSmith | Internet / sign-in | Platforms | Sources |
|---|---|---|---|---|---|
| **Smart Redact** | Scans an image and covers detected sensitive data with blur, pixelate or a solid shape. Nine categories: addresses, card numbers, dates, emails, faces, IP addresses, phone numbers, SSNs, URLs. Categories can be toggled. Started from the Blur tool's Actions panel. | Win: ABBYY OCR plus OpenCV classifier/face detection. Mac: Apple OCR plus Apple face/person detection. No upload to TechSmith or vendors. | Offline (on-device) | Both. Mac only (2026.0.0): click-to-redact and Redact All Matches. Win: accuracy boost via the Optimized OCR experiment. | SR-FEAT, HIDE, BLUR, AI3P, W25, M25, M26, W26 |
| **Grab Text** (OCR) / **Edit Text** | Extracts text from an image (or from a paused video frame on Mac). Edit Text replaces text that is baked into an image. | Win: ABBYY plus Windows Media OCR; Microsoft OCR became the default in 2026.4.0. Mac: Apple OCR. | Offline | Both. Grab Text from video documented only on Mac. | TEXT, AI3P, W26 |
| **Smart Move** | Breaks a screenshot into movable, editable objects (buttons, text). A Detail slider controls granularity. | OpenCV, local; no upload. | Offline | Both. Win: toggle in Tool Properties. Mac: button (since 2024.4.0). | MOVE, HIDE, AI3P, M24 |
| **Simplify / Auto Simplify** | Covers text and UI elements with simplified shapes; Detail slider; custom palettes. | Not listed on the AI page; detection method not stated. (?) | Probably offline (?) | Both. Lock Objects is Windows-only. | SIMP |
| **Background noise removal** | Filters fans, chatter, etc. out of mic audio while recording. | DeepFilterNet (open source), on-device | Offline | Mac since 2024.3.0; Windows since 2025.0.0 | AI3P, M24, W25 |
| **Remove Background** (image) | One-click removal of a photo's background. | Apple Subject Detection | Offline | **Mac only** | RMBG, AI3P |
| **Webcam blur / virtual background** | Blurs or replaces the camera background. | Apple Person Segmentation | Offline | **Mac only**. Intel Macs get blur only, no virtual backgrounds. | PIP, AI3P, M25 |
| **Step Capture** (auto step text) | Screenshot on each click or drag, plus auto-written step text and right-click labels. | Not on the AI page; the text-generation method is not public (?). Win can capture elevated apps. | Offline for capture (?). Cursor stamp pack download needs a TechSmith account. | Both | STEP, W25, M25 |
| **Apply only to text** (Highlight/Blur/Spotlight) | Restricts the effect to detected text runs. | Not stated; probably the OCR engine (?) | Offline (?) | **Mac only** (started as a 2024 experiment, shipped 2026.0.0) | HL, BLUR, M26, M24 |
| **HDR capture / HDR Capture Fidelity** | Color-correct captures on HDR displays. | Not AI | Offline | Windows (Mac histories never mention HDR) (?) | W24, W25 |
| **TechSmith Assets – AI Image Generator** | Generates images from a prompt (beta). | **Cloud, generative**: Azure gpt-image-1-mini | Internet, TechSmith sign-in and a **premium Assets subscription** (included in Camtasia Pro). Run on the Assets website, then sent or imported into the Snagit Library. | Both (web-based) | ASSETS, ASSETS-KB, AI3P, CAMSUB, W24 |
| **Assets browsing** (stamps, templates, themes, photos) | Library of downloadable content. | — | Internet. Windows: the button opens the website, you sign in, then "Send to Snagit". Mac: in-app Assets tab (2025.4.0, photos added 2026.1.0). Admins can strip Assets links (Win 2025.3.1). | Both, with different UX | ASSETS, M25, M26, W25 |
| **Screencast / Share Link** | Uploads the capture and copies a link. Screencast is the default host; viewers can comment. | Cloud | Internet plus a free TechSmith Account. Admins can retarget Share Link to Google Drive, or OneDrive (Windows only). | Both | SHARE, W25, M25 |
| **Camtasia Online** | Share images (2025.4.0), videos (Win 2026.2.0 / Mac 2026.1.0) and Step Captures (2026.3.0); record directly to it (2026.4.0). Collaborative web editor with Looks, backgrounds and narration. | Cloud. Camtasia Online itself uses **cloud generative AI** (text generation, TTS via Azure/ElevenLabs, media analysis); this happens on the web side, not in Snagit. | Internet and an account; works in Chrome or Edge | Both | W25, W26, M25, M26, STEPCO, AI3P |
| **Camtasia Editor / Rev hand-off** (multitrack) | Screen, camera, cursor and audio recorded as separate streams. | Local | No internet needed (?). Exports without a Camtasia subscription carry a watermark. | Both | MULTI, W26, M25 |
| **Cloud share destinations** | Google Drive, Dropbox, Box, OneDrive/SharePoint (Win), Slack, Teams, YouTube | Cloud | Each needs sign-in or OAuth. Slack authorization is stored in the TechSmith Account (Mac). Win OneDrive/SharePoint/Teams use Windows authentication (2026.3.0). | See section 3 | SHARE, M24, M25, W26 |
| **Licensing / subscription** | Subscriptions activate by TechSmith-account sign-in, with no product key. Offline messaging when a subscription has expired. | Cloud check | Sign-in required to activate | Both | STORE, W26, M26 |
| **Experiments and analytics** | Turning on experiments sends anonymous usage data. The tab is hidden if analytics are disabled at deployment (Mac 2024.2.0 note). | — | — | Both | PREF-W, PREF-M, M24 |

---

## 3. Windows vs Mac matrix

Legend: **W** = Windows only, **M** = Mac only, **≠** = on both but behaves differently, **=** = parity reached (version noted).

### Capture

| Feature | Status | Notes | Sources |
|---|---|---|---|
| Panoramic / manual scrolling capture | = (?) | Shipped on both in 2016 (Snagit 13 / Mac 4). Current docs describe it only as manual scroll via the Scrolling Capture button in the All-in-One toolbar. No platform split stated. | BETA16, SCROLL |
| Automatic scrolling capture | = | Scroll arrows appear on scrollable windows. Mac fixes for Safari/Edge and long captures. | SCROLL, M24, M26 |
| Menu capture | ≠ | Both have it. Windows adds Menu Properties (include open submenus / menu bar). Mac has a dedicated hotkey (Ctrl+Shift+E). | COMMON, HELP24, HK |
| Interval (time-lapse) and Scheduled capture | ≠ | Windows had these long before; they were listed Windows-only in the 2024 help. Mac added them in 2026.1.0. Windows minimum interval is 3 s since 2026.3.1. | HELP24, M26, W26, IMGCAP |
| Window capture | ≠ | Windows: crosshair snaps to windows; video records the fixed region. Mac 2026.0.0: Selection > Window follows the window when it moves or resizes. Mac also has Exposé window capture (Ctrl+Shift+W) and Mission Control capture (Ctrl+Shift+M). | VIDCAP, M26, HK, COMMON |
| Freehand-area capture | W | | COMMON, IMGCAP |
| Snagit Printer (capture from the Print dialog) | W | Not available on Windows on ARM. | COMMON, SYSREQ-WEB, ARM |
| Capture Widget (edge tab UI) | W | Alternative to the Capture Window; chosen in Capture Prefs. | PREF-W |
| Explorer shell extension | W | Not installed by default since 2024.3.0. | W24 |
| Whole webpage by dragging a URL to the menu bar or Dock | M | | COMMON |
| Webcam still-image capture | M | | COMMON, HELP24 |
| AirDrop to Snagit; Continuity Camera import; open system screenshots in Snagit | M | | M24, M25, PREF-M |
| Multiple-area capture | ≠ | Windows: Advanced selection menu. Mac: Cmd in All-in-One. | COMMON |
| Repeat last capture | ≠ | Windows: no default hotkey (user assigns one). Mac: Ctrl+Shift+R. | HK, COMMON |
| Multi-monitor full screen | ≠ | Mac selects one screen at a time. | COMMON |
| HDR capture settings | W (?) | | W24, W25, PREF-W |
| Hide Snagit's own toolbars from scrolling/video output | W | Requires Win10 20H1 or later. | W24 |
| Fixed-region capture with exact pixel entry | = | Exact numeric size or position is not possible in the Editor (Mac KB). | COMMON, XYKB |

### Video

| Feature | Status | Notes | Sources |
|---|---|---|---|
| Recording auto-stop timer | M | Mac 2026.4.0 | COMMON, VIDCAP, M26 |
| H.265 recording | M | Mac 2025.2.0 | M25 |
| Retina downsampling | M | | PREF-M, M26 |
| Scale recording to 1920×1080 | W | Capture Prefs option | PREF-W |
| Hide desktop icons while recording | M | | M24 |
| Camera mirror, border, virtual background, blur | M | Virtual backgrounds need Apple Silicon. | PIP, VIDCAP, M25 |
| Automatic mic input gain | M | | VIDCAP, HELP24 |
| 4:3 aspect-ratio lock | W | Both lock 16:9: Ctrl on Windows, Shift on Mac. | VIDCAP |
| Screen Draw Pen tool | M | Arrow, Square and Step are on both. "Screen Draw any time" is a Mac experiment. | DRAW, M25, M26 |
| Separate right-click color in click animation | M | Mac 2025.3.0 | M25 |
| Make Silent Copy; Add Narration to images | M | | M24, WN24 |
| Background noise removal | = | Mac 2024.3.0; Windows 2025.0.0 | M24, W25 |
| 60 fps | = | Mac 2024.1.0; Windows 2024.2.0 | M24, W24 |
| Fullscreen ⇄ PiP camera swap | = | Mac 2024.0.0; Windows 2024.1.0 | WN24 |
| Multitrack to Camtasia | ≠ | Export formats differ: M4A on Windows, MOV on Mac. | MULTI |

### Editor

| Feature | Status | Notes | Sources |
|---|---|---|---|
| Automatic Remove Background | M | | RMBG |
| "Apply only to text" for Highlight/Blur/Spotlight; Highlight shape choice | M | | HL, BLUR, M26 |
| Smart Redact click-to-redact and Redact All Matches | M | Mac 2026.0.0 | M26, CHLOG |
| Gradients (canvas, shape, callout, fill) | M (?) | No Windows release note mentions them. | GRAD, M25 |
| Corner rounding at capture time; multi-object corner radius, resize and rotate | M (?) | Mac 2025.2.1 and 2026.1.0 | M25, M26 |
| Grab Text from a paused video frame | M | | TEXT, HELP24 |
| Live Measure (pixel distance) | M (?) | Only Mac notes mention it. | M24 |
| Lock Objects in Simplify | W | Mac has general object locking instead, which Windows does not honor. | SIMP, COMPAT |
| Auto-Fill "optimize for speed / photo quality" | W | | SEL |
| Print Selection | M | | SEL |
| Smart Move activation | ≠ | Windows: toggle. Mac: button, then Done to flatten. | MOVE |
| Curve and callout properties | ≠ | Windows: Bezier handles, callout padding and shading. Mac: different handles, No Line outline, rounded radii, dashed shapes. Some properties are lost when files move between platforms. | COMPAT, HELP24 |
| Assets access | ≠ | Windows: website. Mac: in-app tab. | ASSETS |
| Tray pinning | = | Windows 2024.0.0; Mac 2026.1.0 | W24, M26, WN-TUT |
| Select All like objects | = | Windows 2026.1.0; Mac 2025.3.0 | SIMP, W26, M25 |
| Plain-text copy from Grab Text | = | Windows 2024.3.5 | W24, TEXT |

### Step Capture

| Feature | Status | Notes | Sources |
|---|---|---|---|
| Outputs on both platforms | ≠ | Word: Windows 2025.3.0, Mac 2025.1.0. PowerPoint: Windows 2026.1.0, Mac 2026.0.0. | STEPOUT, OFFICE-KB |
| Outputs on Mac only | M | Pages, Keynote, Markdown, HTML, batch export of separate images | STEPOUT |
| Scale Image effect | W | | STEP |
| New Capture from a drop zone; lock icon before editing | M | | STEP |
| Recommended step ceiling | ≠ | About 50 steps on Windows, about 100 on Mac | STEP |

### Share destinations (SHARE)

| Destinations | Status |
|---|---|
| Email, OneNote, Evernote, SharePoint, OneDrive (incl. as Share Link target), Program, Clipboard, Printer | W |
| X/Twitter | Was W; removed in Windows 2026.4.0 |
| Outlook, Pages, Keynote, Numbers, Application, macOS share-sheet services (AirDrop, Mail, Messages, Notes, Photos, Freeform, Journal, Reminders) | M |
| Word, Excel (images); PowerPoint (images and video); Teams; Slack; Google Drive; Dropbox; Box (not shown by default on Mac before 2026.3.0); YouTube; Screencast; File; FTP; Camtasia Editor; Camtasia Online | Both |
| Knowmia | Removed on both (2025) |
| Panopto | Removed: Windows 2024.3.3, Mac 2026.0.0 |

Sources: SHARE, W24, W26, M26.

### Default hotkeys (HK)

| Action | Windows | Mac |
|---|---|---|
| Capture | Print Screen | Ctrl+Shift+C |
| Start / pause recording | Shift+F9 | Ctrl+Shift+Space |
| Stop recording | Shift+F10 | Ctrl+Shift+V (also Video Capture) |
| Exit Capture | Ctrl+Shift+X | — |
| Toggle Screen Draw | Ctrl+Shift+D | not stated |
| Fullscreen ⇄ PiP camera | Ctrl+Shift+F (page gives one shortcut; per-platform modifier not stated) | Ctrl+Shift+F (same note) |
| Mac-only defaults | — | All-in-One Ctrl+Shift+O; Image Ctrl+Shift+S; Mission Control Ctrl+Shift+M; Window Ctrl+Shift+W; Menu Ctrl+Shift+E; Repeat Ctrl+Shift+R |
| Editor tool selection | Ctrl+1…9 | Control+letter |

Sources: HK, DRAW, PIP.

### Settings and deployment

| Feature | Status | Notes | Sources |
|---|---|---|---|
| Preferences layout | ≠ | Windows splits Capture Preferences and Editor Preferences (Experiments tab is in Editor Prefs). Mac has a single Settings window. | PREF-W, PREF-M |
| Mac App Store edition | M | Perpetual 2024 build (2024.4.10), supported until 31 Dec 2026 | MAS |
| Enterprise deployment | ≠ | Windows: MSI/EXE, Enterprise Deployment Tool, MST. Mac: Support Tool, plus a new deployment tool in 2026.4.0. | W24, NEO2600, M26 |

---

## 4. System requirements (current version)

### Windows
Sources: SYSREQ-WEB (modified 17 Sep 2026); SYSREQ-KB (updated 5 Dec 2025); WINVER; ARM.
- **OS:** Windows 11; Windows 11 on ARM 23H2+; Windows 10 20H2+; Windows Server 2016, 2019 and 2022.
  - 64-bit only.
  - N editions need the Media Foundation Pack.
- **Conflicts between sources:**
  - The Dec 2025 KB lists Win10 **1903+**, Win11 ARM **22H2+** and Server 2016/2019 only. The newer web page is stricter.
  - WINVER says Server 2022 needs Snagit **2026.1.1** or later.
- **CPU:** Intel Core i5 6th gen or AMD equivalent. **Video recording needs Intel 8th gen+ or AMD Ryzen 2000+.**
- **ARM64:** No native ARM64 build. Snagit runs under x64 emulation and TechSmith says it has no plans for a native build. The Snagit Printer driver does not work on ARM. (KB updated 16 Jan 2025)
- **RAM:** 8 GB recommended; no minimum is published.
- **Disk:** 1.6 GB for installation.
- **Runtimes:** .NET Framework 4.7.2 or later. WebView2 Runtime (bundled with the installer).
- **UI languages:** English, French, German, Japanese, Portuguese, Spanish (PREF-W).

### macOS
Sources: SYSREQ-WEB; SYSREQ-KB; MACVER (updated 24 Sep 2026); M25; M26; AS; ROSETTA; ARS; UNSUP; MAS.
- **OS:** macOS 27, 26 (Tahoe) and 15 (Sequoia).
  - macOS 27 requires Snagit **2026.3.3** or later.
  - macOS 26 requires 2025.3.1 or later (or 2024.4.7 or later).
  - **macOS 14 dropped in Snagit 2026**; the last build for it is 2025.4.0. macOS 13 was dropped in 2025; macOS 12 ended with 2024.2.5.
- **CPU:** Universal app, native on Apple Silicon since Mac 2021.3.0 (AS). A community site reports the Universal binary at 2026.3.3 (ROSETTA). (?)
  - Intel Macs are still supported on 15/26: Mac 2026.0.1 includes an Intel-specific fix. Webcam virtual backgrounds are not available on Intel (PIP).
  - macOS 27 itself requires Apple Silicon (Ars Technica, via ARS), so Snagit on 27 is effectively Apple Silicon only.
- **RAM, disk, display:** Not published by TechSmith for Mac. (?) The Mac App Store 2024 build is 178.4 MB and needs macOS 13+ (MAS).
- **Unsupported setup:** macOS patched onto hardware Apple doesn't support (UNSUP).
- **Languages:** Same six as Windows (MAS).

---

## 5. Experiments tab (publicly described)

How it works:
- **Windows:** Editor Preferences > Experiments.
- **Mac:** Snagit Settings > Experiments (2024 notes). The 2026.3.0 Mac notes call it "Editor Preferences".
- Each experiment can be toggled and has a **Send Feedback** button. TechSmith says experiments may change or vanish, get no tech support, and collect anonymous usage data (PREF-W, PREF-M).
- Mac hides the tab when analytics are disabled at deployment (M24 2024.2.0).
- The tab appears on **Mac from 2024.1.0**. Windows notes first list experiments in **2025.4.0** (WN24, W25). (?)

**Experiments most likely still current (Oct 2026):**

| Platform | Experiment | Introduced | Status | Sources |
|---|---|---|---|---|
| Windows | **Improved / Optimized OCR** (better find-and-edit of text; improves Smart Redact accuracy) | 2026.0.0, extended 2026.3.0 | Microsoft OCR became the default engine in 2026.4.0, so it may have graduated. (?) | W26, NEO2600, NEO2630 |
| Windows | **Quick Image Annotation**: capture, annotate and copy to the clipboard without opening the Editor | 2026.1.0 | Current | W26 |
| Windows | **HDR Capture Fidelity**: colors matched to HDR displays | 2025.4.0 | Current (?) | W25, NEO2540, WN25 |
| Mac | **Screen Draw available any time**, not just while recording (useful for screen sharing) | 2026.3.0 | Current | M26 |
| Mac | **QR codes**: make custom codes, get a code for a shared capture, scan codes during capture | 2025.4.0 | Current (?) | M25 |
| Mac | **Full-screen capture with each window as a separate editable layer** | 2025.0.0 | Status unknown (?) | M25 |
| Mac | **Capture Hotkeys in Video**: shows pressed keys and combos in the recording; toggled under Screen Draw options | 2024.2.0 | Status unknown (?) | M24, MRN24 |

**Earlier Mac experiments that later shipped:**
- Sensitive-data auto-blur (2024.2.3) became **Smart Redact** (2025.0.0).
- Smart text selection and Smart Highlight (2024.1.0) became **Apply only to text** (2026.0.0).
- Canvas and floating-image corner rounding (2024.1.0 / 2024.3.0) became **Rounded corners** (2024.4.0).
- Pen Screen Draw (2024.1.0) became a standard Pen tool (2025.0.0). Oval Screen Draw status is unclear. (?)
- Single moving-window recording (2025.0.0) became **Window Capture** (2026.0.0).
- Pixel distance measurement (2024.1.0) appears as Live Measure in later notes. (?)

Sources: M24, M25, M26.

---

## URLs consulted (deduplicated)

**Version histories and release notes**
- W24: https://support.techsmith.com/hc/en-us/articles/39102250364429-Snagit-Windows-2024-Version-History
- W25: https://support.techsmith.com/hc/en-us/articles/39102293598349-Snagit-Windows-2025-Version-History
- W26: https://support.techsmith.com/hc/en-us/articles/42674936732685-Snagit-Windows-2026-Version-History
- Windows history index: https://support.techsmith.com/hc/en-us/articles/115006435067
- M24: https://support.techsmith.com/hc/en-us/articles/37938485483021-Snagit-Mac-2024-Version-History
- M25: https://support.techsmith.com/hc/en-us/articles/37938520706957-Snagit-Mac-2025-Version-History
- M26: https://support.techsmith.com/hc/en-us/articles/41975263481613-Snagit-Mac-2026-Version-History
- Mac history index: https://support.techsmith.com/hc/en-us/articles/115006443147
- WN26: https://download.techsmith.com/update/snagit/enu/26_updatemessage.html
- WN25: https://download.techsmith.com/update/snagit/enu/25_updatemessage.html
- MRN24: https://download.techsmith.com/update/snagitmac/enu/2420/rnotes.html
- CHLOG: https://www.techsmith.com/snagit/whats-new/
- CHLOG entry: https://techsmith.com/snagit/whats-new/capture-vibrant-colors-with-hdr-image-support
- WN-TUT: https://www.techsmith.com/learn/tutorials/snagit/whats-new/
- WN24: https://techsmith.com/blog/tutorials/snagit/whats-new

**AI, redaction and editing**
- AI3P: https://support.techsmith.com/hc/en-us/articles/26140574750861
- SR-FEAT: https://www.techsmith.com/snagit/features/smart-redact/
- HIDE: https://www.techsmith.com/learn/tutorials/snagit/hide-sensitive-information
- BLUR: https://www.techsmith.com/learn/tutorials/snagit/blur/
- HL: https://www.techsmith.com/learn/tutorials/snagit/highlighter/
- MOVE: https://www.techsmith.com/learn/tutorials/snagit/move/
- SIMP: https://www.techsmith.com/learn/tutorials/snagit/simplify/
- TEXT: https://techsmith.com/learn/tutorials/snagit/edit-image-text
- RMBG: https://www.techsmith.com/learn/tutorials/snagit/remove-background/
- SEL: https://www.techsmith.com/learn/tutorials/snagit/selection-tools/
- GRAD: https://techsmith.com/learn/tutorials/snagit/gradients
- Feature category pages: https://www.techsmith.com/sngfcategory/editing/ and https://www.techsmith.com/sngfcategory/screenshots/

**Step Capture and Camtasia**
- STEP: https://www.techsmith.com/learn/tutorials/snagit/step-capture/
- STEPOUT: https://www.techsmith.com/learn/tutorials/snagit/step-capture-outputs/
- STEPCO: https://www.techsmith.com/learn/tutorials/snagit/step-capture-to-camtasia-online/
- MULTI: https://www.techsmith.com/learn/tutorials/snagit/multitrack-recording/
- CAMSUB: https://support.techsmith.com/hc/en-us/articles/41227202394637-Camtasia-Individual-Subscriptions-Now-with-Snagit

**Capture, video and sharing**
- SHARE: https://www.techsmith.com/learn/tutorials/snagit/share/
- OFFICE-KB: https://support.techsmith.com/hc/en-us/articles/360029399471-Share-to-Word-PowerPoint-Excel-or-OneNote-from-Snagit-Video
- HK: https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/
- COMMON: https://www.techsmith.com/learn/tutorials/snagit/common-captures/
- IMGCAP: https://techsmith.com/blog/tutorials/snagit/image-capture
- HOWCAP: https://www.techsmith.com/learn/tutorials/snagit/how-to-capture-your-screen/
- VIDCAP: https://www.techsmith.com/learn/tutorials/snagit/how-to-capture-video/
- RECED: https://techsmith.com/learn/tutorials/snagit/record-edit-share-snagit-video
- PIP: https://www.techsmith.com/learn/tutorials/snagit/picture-in-picture/
- DRAW: https://www.techsmith.com/learn/tutorials/snagit/screen-draw/
- SCROLL: https://techsmith.com/tutorials/snagit/scrolling-capture
- Capture Window webinar: https://www.techsmith.com/learn/webinars/learn-one-new-thing-the-snagit-capture-window/

**Assets, preferences and file compatibility**
- ASSETS: https://www.techsmith.com/learn/tutorials/snagit/assets/
- ASSETS-KB: https://support.techsmith.com/hc/en-us/articles/37458077453581-What-Are-TechSmith-Assets-for-Snagit
- PREF-W: https://techsmith.com/learn/tutorials/snagit/preferences
- PREF-M: https://www.techsmith.com/blog/tutorials/snagit/preferences-mac/
- COMPAT: https://www.techsmith.com/blog/tutorials/snagit/version-compatibility/
- HELP24: https://techsmith.com/blog/wp-content/uploads/2025/01/Snagit-Help-2024-EN.pdf
- XYKB: https://support.techsmith.com/hc/en-us/articles/45792608030861-Can-I-Set-the-Exact-Position-or-Size-of-Objects-or-Selections-in-Snagit-Editor

**System requirements and compatibility**
- SYSREQ-WEB: https://techsmith.com/snagit/system-requirements and https://techsmith.com/snagit-system-requirements.html
- SYSREQ-KB: https://support.techsmith.com/hc/en-us/articles/41688606378637-What-Are-the-System-Requirements-for-Snagit
- WINVER: https://support.techsmith.com/hc/en-us/articles/219908907
- MACVER: https://support.techsmith.com/hc/en-us/articles/219910027
- ARM: https://support.techsmith.com/hc/en-us/articles/27938424236685-Snagit-ARM-Compatibility-and-Emulation-Details
- AS: https://support.techsmith.com/hc/en-us/articles/360052555611-Snagit-and-Apple-Silicon-Compatibility-Info
- UNSUP: https://support.techsmith.com/hc/en-us/articles/36280774958605-Unsupported-Environments-Using-Snagit-on-macOS

**Store, press and third-party**
- STORE: https://techsmith.com/store/snagit/upgrade
- MAS: https://apps.apple.com/us/app/techsmith-snagit-2024/id6451154048
- PR25: https://www.reworked.co/the-wire/snagit-and-camtasia-2025-introduce-ai-and-screentelligence-powered-workflows-for-faster-more-impactful-content-creation/
- NEO2600: https://www.neowin.net/amp/snagit-20260/
- NEO2630: https://www.neowin.net/amp/snagit-202630/
- NEO2540: https://www.neowin.net/amp/snagit-202540/
- BETA16: https://betanews.com/2016/06/07/snagit-13-for-windows-and-snagit-4-for-mac-released-introduce-panoramic-capture-and-animated-gif-tools/
- ROSETTA: https://rosettacheck.com/apps/com.TechSmith.Snagit
- ARS (Ars Technica summary): https://tagteam.harvard.edu/hub_feeds/3382/feed_items/18328372/about
- VideoHelp: https://videohelp.com/software/Snagit/version-history

Tried but could not read: https://www.neowin.net/software/snagit-202630/ (fetch error) and https://www.kandji.io/library-items/snagit-2025 (redirected; not followed).
