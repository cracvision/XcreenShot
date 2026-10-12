# Snagit parity inventory: Library, Sharing/Outputs, Presets/Themes, Preferences, Integrations, Experiments, Licensing

**How this was built.** I used only public TechSmith tutorials, support articles, version histories (Windows 2019–2026, Mac 2019–2026), deployment and COM guides, archived help PDFs and a few reviews or changelog mirrors. No binaries, installers or .snagx files were touched. All descriptions are paraphrased.

**Coverage limits:**
- The shared WebSearch budget (200 per turn) ran out near the end. Two planned searches did not run: Snagit Printer settings and the Snipping Tool integration. Both topics are still covered below from pages fetched earlier.
- No public AppleScript or Shortcuts dictionary for Snagit for Mac was found.
- No dedicated public article on Share History was found. That item is pieced together from release notes and older help files.

**Platform:** Win = Windows, Mac = macOS, Both, or Unknown. **Version:** first release where a source states the feature (W = Windows build number, M = Mac build number). **(?)** means uncertain or only implied. Source keys such as [LIB] are expanded in the URL list at the end.

---

## 1. Library

### 1.1 Core and organization

| Feature | What the user can do | Platform | Version | Sources |
|---|---|---|---|---|
| **Library (tab/view)** | Full capture history (images, videos, GIFs) inside the Editor. Switch between Editor and Library views (Mac: Cmd+1 for Editor, Cmd+2 for Library). Double-click a thumbnail to open it in the Editor. | Both | Long-standing; Editor and Library toggle placed on the toolbar W2025.3.0 / M2025.2.0 | [LIB] [HOTK] [M25] [WN] |
| **Search pane categories** | Filter by: file type (Images, Videos, Step Captures; Animated GIFs on Mac only); date (year, month, day); Applications (source app); Websites (source URL/domain); Tags; Favorites (Mac only). | Both (differences noted) | Day/month/year browsing restored W2022.0.2; date and website filters M2022.1.0; video metadata in Applications/Websites W2023.2.0; Step Capture category 2026.3.0 (both) | [LIB] [W22] [M22] [W23] [W26] [M26] |
| **Search field** | Find captures by file name, and also by tag, application or website. Mac also matched annotation text (2021-era help) (?). Result counts shown for accessibility (W2025.1.0). Mac search up to 3x faster (M2025.2.0). | Both | — | [LIB] [H21] [W25] [M25] |
| **Search depends on OS indexing (Win)** | Windows Library listing and search rely on Windows Search indexing. If the Library folder or .snagx files are not indexed, the Library looks empty. A fallback for bad indexing was added W2022.0.1. | Win | 2022+ | [SEARCH] [NODISP] [W22] |
| **Sort** | Sort by date modified, date created, name or size. Click a column header; click again to reverse. | Both | — | [LIB] [LIBLOC] |
| **Thumbnail size / zoom** | Win: zoom slider and percentage; zoom level remembered (several fixes). Mac: zoom slider; zoom level persists (M2019.1.8). | Both | — | [LIB] [W20] [W21] [W25] [M19] |
| **Tags** | Multi-select, then use the Tag button to type a new tag or pick an existing one. Remove: Win right-click > Delete Tag; Mac Delete key. Tags are visible only inside Snagit. Videos taggable W2024.0.3. Mac has Image > Edit Tags (M2023.2.0). Tags that differ only by capitalization merge when filtering (M2025.4.0). | Both | — | [LIB] [W24] [M23] [M25] |
| **Favorites (capture-level)** | Mark captures as favorites and filter by them. Marking a favorite does not change the modified date (M2023.2.6). | Mac | ≤2022 (?) | [LIB] [M23] [M26] |
| **Flags, keywords, folder view (legacy)** | Snagit 10–12 had colored flags shown as thumbnail badges, user keywords, a folder-tree view, and a "recent captures by week" view. Not present in current versions; treat as historical. | Win (legacy) | Snagit 10–12 | [H10] [H12] |
| **Details / Get Info** | Show capture metadata: original capture time, source app, URL, OS. Win: right-click > Details. Mac: Get Info (not available for multi-selection). | Both | Original capture time shown W2022.0.1 | [LIB] [W22] [M22] |
| **Thumbnail tooltips with file details** | Hover a thumbnail to see file details. | Mac (Win ?) | M2024.0.0 | [M24] [RNM24] |
| **Quick Look preview** | Space bar previews the selected item; arrow keys move through items. | Mac | M2024.0.1 | [RNM24] [M26] |
| **Reveal in Library** | Jump from the open capture to its Library entry (File menu; Shift+Cmd+L). | Mac | M2022.2.0 | [M22] [HOTK] |
| **Delete / Move to Trash** | Delete single, multiple or a range of items (Shift-click). Optional "Prompt before deleting files" preference (Win). | Both | — | [LIB] [PREFW] |
| **Create from Library** | Create button: Image from Template, or Video from Images, using the selected captures. | Both | Templates 2020 | [LIB] [W20] |
| **Print from Library** | Print from the context menu. | Mac (Win ?) | M2020.2.1 | [M20] |
| **Batch Export / Batch Conversion Wizard** | Apply one format, effects and naming to many images. Win: multi-step wizard (add files, effects, folder, format, automatic file name with prefix and numbering), started from Library right-click or File > Export. Mac: single dialog (base filename with numeric suffix, effect with settings, rotation, folder, format, Reveal in Finder). Windows Explorer context-menu entry exists as an installable component. | Both | Mac batch export M2020.2.0 | [BATCH] [M20] [DEPW] |
| **Step Capture items in Library** | Step Captures get their own category and show an image-count badge on thumbnails. Mac can export them from the Library. | Both | 2026.3.0; Mac export fix M2026.4.0 | [W26] [M26] |
| **Translate (localization)** | Export annotation and callout text to an XLIFF 2.0 file, import translations, generate localized copies tagged by project and language. Removed from Mac in 2024.2.5. | Mac (also listed in Win 2020/2021 help) | Removed M2024.2.5 | [TRANS] [H20] [H21] |
| **Large-library handling** | Thumbnails render faster and more reliably with more than 5,000 items (W2026.1.0). Better load and responsiveness for large or cloud libraries (M2024.3.0, M2024.4.0, M2026.0.1). | Both | — | [W26] [M24] [M26] |
| **Library keyboard navigation** | Select all (Cmd+A), jump to top/bottom (Cmd+Up/Down), context menu via Control-click, Find Capture (Cmd+F). | Mac | M2019.1.10 / M2020.1.3 | [M19] [M20] [HOTK] |
| **Copy All from Library** | Copy the selected Library item to the clipboard. | Mac | — | [LIB] |

### 1.2 Auto-save, storage and location

| Feature | Behavior | Platform | Version | Sources |
|---|---|---|---|---|
| **Automatic Library save** | New image captures auto-save as .snagx in the Library. Unsaved videos auto-save as MP4. Win has separate on/off toggles for images and videos. Mac always saves to the Library. | Both (toggles Win only) | SNAGX since 2022 | [SAVE] [PREFW] [TRAY] |
| **SNAGX format** | Cross-platform editable project format that replaced .snag (Win) and .snagproj (Mac). | Both | 2022.0.0 | [W22] [M22] [SAVE] |
| **Default Library location** | Win: Documents\Snagit. Mac: Pictures/Snagit. Mac setting was formerly called "Autosave Location". | Both | — | [LIBLOC] [PREFM] |
| **Change Library location** | Win: Editor Preferences > Library > Choose location (a tooltip shows the path; "Browse other" for any folder). Mac: Settings > Advanced > Choose. Snagit moves all files, is briefly unavailable, and closes all tray items. Admins can lock the location. | Both | — | [LIBLOC] [DEPW] |
| **Cloud Library** | Keep the whole Library in a synced folder: Box (Win via Box Sync, Mac via Box Drive offline), Dropbox, Google Drive (desktop app in "Mirror" mode), iCloud, OneDrive Personal and Business (files-on-demand off). Snagit auto-detects locally installed and actively syncing clients (one account per service). If a Library already exists there, Snagit suggests it. Large-library size warning on Mac. | Both | 2022.0.0; iCloud on Mac M2022.1.0 | [LIBLOC] [W22] [M22] |
| **Network location** | Supported with limits: no file counts; no date, tag, app or website filters; file-name search only. | Win | W2022.1.0 | [LIBLOC] [LIB] [W22] |
| **No first-run location prompt** | Snagit stopped asking for a Library location on first capture. | Both | W2024.0.0 / M2025.2.1 | [W24] [M25] |
| **Loose PNG/JPG shown** | PNG and JPG files saved into the Library folder appear in the Library. | Win | W2024.1.0 | [W24] |
| **Library status message** | Status banner stays expanded by default and remembers its state. | Win | W2023.1.0 | [W23] |
| **Save As vs Library** | Save As writes a copy in another format or place; the .snagx stays in the Library. Mac flattens non-SNAGX files on quit. "Keep a duplicate editable capture when saving" option (Mac). | Both | — | [SAVE] [PREFM] |

### 1.3 Maintenance, migration, import/export

| Feature | Behavior | Platform | Version | Sources |
|---|---|---|---|---|
| **Convert previous captures** | Prompts to convert .snag to .snagx; can be ignored and done later; optional .snagproj conversion on Mac. Old-format backups go to the DataStore folder (Win) or a "library-conversion-backup" folder (Mac). "Show prompt to convert" preference. | Both | 2022+ | [CONV] [PREFW] |
| **Import Backup (.snagarchive)** | Import archives made by Snagit 9.1–2021. Only SNAG, MP4 and GIF are imported; SNAG files are converted. Button re-added W2026.3.0, with an explanation when it is disabled. | Win | W2022.1.3 / W2023.0.0 | [MANLIB] [W22] [W23] [W26] |
| **Legacy Backup/Restore (≤2021)** | Create Backup made a .snagarchive. Restore replaced the whole Library (no merge). Datastore folder was movable from 12.4.1 on. | Win | ≤2021 | [BACKUP] [EMPTY] [SAVED21] |
| **Move Library to a new computer** | Copy the Snagit Library folder, then point the new install to it with Choose > Browse other. | Both | 2022+ | [MANLIB] [BACKUP] |
| **Merge libraries** | Pointing Snagit at another Library folder merges the two (Win 2022+ and Mac); then move it back if wanted. | Both | 2022+ | [MANLIB] |
| **Export Library on exit** | Batch-convert .snagx to PNG/JPG; videos and GIFs are already plain files. | Both | — | [EXPORTLIB] |
| **IT library conversion tool** | Admin tool for converting older Libraries. SQL CE components are required for pre-2022 conversion. | Win | W2024.1.0 | [W24] [DEPW] |
| **Support Tool repoint** | Support Tool can point Snagit to a different Library folder and fix permissions. | Mac (Win ?) | M2026.0.1 | [M26] [M23] |
| **Mobile import to Library** | Fuse, AirDrop and Continuity Camera items land in the tray/Library (see §8). | Both/Mac | — | [MOBILE] |

---

## 2. Recent Captures Tray

| Feature | Behavior | Platform | Version | Sources |
|---|---|---|---|---|
| **Show/Hide tray** | Win: "Show Recent / Hide Recent" button. Mac: "Recent" button, Option+Cmd+T. | Both | — | [TRAY] [HOTK] |
| **Capacity** | User-set count of 1–200 items (was 1–15 or 18 in older versions). | Both | M2022.2.0 / W2024.0.3 | [PREFW] [PREFM] [M22] [W24] [H18W] |
| **Pin / unpin** | Pin keeps a thumbnail anchored on the left; new captures appear to its right. Shortcut Ctrl+Shift+P (Win) / Control+Shift+P (Mac). Multi-pin supported. | Both | W2024.0.0 / M2026.1.0 | [TRAY] [W24] [M26] [HOTK] |
| **Reorder by drag** | Drag pinned or unpinned items into a new order. | Both | W2024.0.0 / M2026.1.0 (improved M2026.3.2) | [TRAY] [M26] |
| **Rename** | Rename via right-click, F2 or the File menu (Win). File > Rename (Mac). | Both | W2023.0.0 / M2024.0.0 | [W23] [M24] |
| **Thumbnail size** | Small, Medium or Large via right-click. | Win | — | [TRAY] [PREFW] |
| **Close / Close Others / auto-clear** | Close removes an item from the tray without deleting the file. Mac has Close Other Captures and Close All (Option+Cmd+W). Win option removes unpinned items when the Editor closes. | Both | — | [TRAY] [PREFW] [HOTK] |
| **Delete** | Delete (Win) or Move to Trash (Mac) permanently removes the file. Ctrl+Delete / Cmd+Delete. | Both | — | [TRAY] [HOTK] |
| **Duplicate** | Duplicate a file (Ctrl+Shift+D / Shift+Cmd+D). Mac can duplicate several selected items at once. | Both | Multi on M2026.3.0 | [HOTK] [M26] |
| **Unsaved indicator** | Orange "sunburst" badge marks unsaved changes. Mac adds a tooltip on the icon. | Both | — | [TRAY] [M26] |
| **Combine** | Drag an image onto the canvas to combine images. Combine Videos from the tray context menu or the Add Videos button. | Both | Combine videos 2023.2.0 | [TRAY] [W23] [M23] [NEO2400] |
| **Multi-select actions** | Create Image from Template, Create Video from Images. | Both | — | [TRAY] |
| **Navigation** | Left/Right arrows (Win) or Cmd+[ / Cmd+] (Mac) step through items. Trackpad and mouse scrolling on Mac (M2025.1.0). | Both | — | [HOTK] [M25] |
| **Drag out to other apps** | Drag a tray item into another app. Default file type for drags is set in preferences (see §5). Video drags keep edits (M2024.2.1). | Both | — | [GTK] [W24] [M24] |
| **Details/Get Info, tags on tray items** | Same as Library. | Both | — | [TRAY] |
| **Move to Library view** | Closed items remain findable in the Library. | Both | — | [TRAY] |

---

## 3. Sharing and outputs

### 3.1 Sharing framework

| Feature | Behavior | Platform | Version | Sources |
|---|---|---|---|---|
| **Share Link button** | One click uploads the selected file and copies a URL to the clipboard. Default target is Screencast. Disabled when nothing is selected, several items are selected, a share is in progress, or (Mac) the format is unsupported. Mac formats: SNAGX/PNG/JPG/GIF/TIFF/BMP/WEBP and MP4/MOV. Progress bar plus "link copied" notice; upload can be cancelled; tooltips explain why it is disabled (M2026.3.2). Shortcut Ctrl+Shift+L / Option+Cmd+L; customizable on Mac (M2023.1.0). If the Screencast plugin is absent, a plain Share button is shown instead. | Both | 2023.0.0 | [LINK] [W23] [M23] [M24] [M26] [HOTK] |
| **Customizable Share Link target (admin)** | Admins can redirect Share Link to OneDrive (Win only), Google Drive, or disable it. Users see a one-time explanation message. Camtasia Online is selectable from the Share Link dropdown (M2026.1.0). | Both | 2025.0.0 | [LINK] [W25] [M25] [DEPW] [DEPM] [M26] |
| **Share dropdown** | Lists enabled destinations. Destinations that cannot take the selected file type are greyed out. Also holds Preferences, Save As Preferences, Share History and (Win) Program > Manage. Screen-reader accessible (W2024.2.0). Win menu bar has Share > Email > Send to Email (W2026.3.0). | Both | — | [SHARE] [W24] [W26] |
| **Enable/disable destinations** | Win: checkboxes in Editor Preferences > Share, applied without restart (W2024.2.0). Mac: Settings > Share with +/– buttons and drag ordering. | Both | — | [SHARE] [PREFW] [PREFM] [W24] |
| **Multiple instances per destination** | Add the same destination several times (for example, separate accounts or Slack workspaces), each with its own Share Name. | Mac | — | [PREFM] [SLACK] |
| **Per-destination options** | Account sign-in/out, privacy level, default folder, "ask for file name each time". Mac adds a custom display name and a keyboard shortcut per destination (shortcuts since M2019.1.2). | Both (name and shortcut Mac only) | — | [SHARE] [LINK] [H23] [M19] [M24] |
| **Capture source in names/titles** | Option to leave the source app or site out of shared file names and titles; date/time or a custom name is used instead. | Both | W2024.1.0 / M2023.3.1 | [SHARE] [PREFW] [W24] [M23] |
| **Multi-file share** | Share several selected items in one action (File, Camtasia, Teams drag, etc.). | Both | M2020.1.2; File multi M2023.2.0; Camtasia multi W2025.4.0 | [M20] [M23] [W25] |
| **Share from Capture / presets** | A preset or the Capture Window can send a capture straight to a destination, optionally without opening the Editor. Win can send to multiple destinations (fixed W2021.4.0). Older Mac had "Add another destination". | Both | — | [SHARE] [SPEED] [W21] [HM4] |
| **Status and notifications** | Shows "Sending..." for local targets and "Uploading..." for cloud targets (W2026.4.0). Completion notices use Windows notifications (W2025.0.0). Screencast notification settings (2024.0.0). | Both | — | [W26] [W25] [W24] [M24] |
| **Share History** | Window listing past shares. You can open the shared item or location, Copy Link, Open Link, and remove entries (removed entries stay removed after restart, W2024.1.2). Open is hidden if the local file moved (M2021.1.0). YouTube entries expire after 28 days (M2023.2.0). Thumbnails and sorting. Mac shortcut Option+Shift+S. Some destinations (for example OneDrive) store the uploaded link there. | Both | Present by 2018 Mac help; Knowmia links W2020.1.5 | [HM4] [W20] [W24] [M21] [M23] [M25] [HOTKM] [PERM] |
| **Toolbar share buttons** | Add any destination to the Editor toolbar via More > Customize Toolbar for one-click sharing. Custom toolbar destinations survive upgrades (W2020.1.4). | Both | — | [TOOLBAR] [LINK] [W20] |
| **Downloadable destination plugins** | Win destinations ship as plugin files (.tscplugin for 2018–2023; .snagacc for Snagit 13) that can be reinstalled individually. | Win | — | [PLUG] [MISSING] |
| **Mac third-party share extensions** | Snagit's Share menu can show macOS share extensions chosen in System Settings > Extensions. | Mac | Snagit for Mac 2018+ (?) | [H18M] [CUST] [H21] |
| **Retina downscale on share** | Mac shrinks retina images when sharing by default; hold Shift while dragging to skip this. "Default sharing format" setting. | Mac | — | [PREFM] |
| **Save As Preferences** | Either remember the last-used format, or always use a chosen format. Win can also set a default type on the Capture Window Image tab. | Both | — | [PREFW] [SAVE] |

### 3.2 Destinations (current and historical)

| Destination | Behavior / options | Media | Platform | Version notes | Sources |
|---|---|---|---|---|---|
| **File** | Save to a chosen folder and format. Step Captures can be saved as separate images (M2026.3.0). Mac fixed format choice on macOS 14. | Img+Vid | Both | — | [SHARE] [M26] [M23] |
| **Clipboard** | Copy the image to the Windows clipboard. Old preference strips transparent areas. | Img | Win | — | [SHARE] [H18W] |
| **Printer** | Print the image. | Img | Win | — | [SHARE] |
| **Email** | New message with the image attached or embedded. Preset subject/body (fixed W2020.1.0). Compose options (W2026.3.1). Works with classic and New Outlook (W2026.2.0); needs a supported client such as Outlook 2016+/365. Better display of wide images (W2026.4.0). | Img | Win | — | [SHARE] [EMAIL] [W20] [W26] |
| **Microsoft Outlook** | Attach to a new email. | Img+Vid | Mac | — | [SHARE] |
| **Program** | Send to any .exe. Program > Manage > Add Program: display name, output type, exe path; parameters auto-filled. | Img+Vid | Win | — | [SHARE] [H23] |
| **Application** | Send to any Mac app (Share Name + Choose Application). Supports apps that accept all file types (M2021.3.0). | Img+Vid | Mac | M2020.2.0 | [SHARE] [M20] [M21] |
| **FTP** | Upload to an FTP server. Options include passive mode and (Mac) a custom port (M2020.1.2). COM-level options include automatic naming and a sequence limit (Win) (?) for the UI. | Img+Vid | Both | — | [SHARE] [FTP] [M20] [COM25] |
| **Microsoft Word** | Insert an image (Win). Mac inserts at the cursor (M2024.4.1) with a choice of new doc, current doc or always ask (M2026.3.2/2026.4.0). Step Capture guides arrive as editable text plus images (W2025.3.0 / M2025.1.0). | Img | Both | — | [SHARE] [M24] [M26] [STEPOUT] |
| **Microsoft PowerPoint** | Insert onto a slide, with a "current slide" option. Video support (W2023.2.1). Step Captures become slideshows with a title slide and editable text (W2026.1.0 / M2026.0.0). | Img+Vid | Both | — | [SHARE] [W23] [M26] [OFFICE] [STEPOUT] |
| **Microsoft Excel** | Insert into a workbook. | Img | Both | — | [SHARE] |
| **Microsoft OneNote** | Insert onto a page. Requires desktop OneNote as the protocol handler. | Img | Win | — | [SHARE] [ONENOTE] |
| **Microsoft SharePoint** | Upload and copy link. | Img+Vid | Win | W2020.1.1 | [SHARE] [W20] |
| **OneDrive** (Business; Personal) | Upload with a default folder option; Stream playback for videos. Delegated file read/write permission; tokens stored encrypted locally; creates a "TechSmith" folder. Windows-native auth (W2026.3.0). | Img+Vid | Win | Personal W2023.2.0/W2024.0.0 | [SHARE] [PERM] [W23] [W24] [W26] |
| **Microsoft Teams** | Dialog toggles between Teams/channel and Chats, picks team + channel or a chat, optional message, Share; link copied. Drag-and-drop into Teams also works. Tenant admin must pre-consent to the app. | Img+Vid | Both | 2024.0.0 | [TEAMS] [PERM] [W24] [M24] |
| **Slack** | Choose workspace, Channels or DMs, recipient, optional message; link copied. Official Snagit Slack app gives inline preview and playback. Workspace authorization is stored with the TechSmith account and syncs across devices (M2024.4.2 / M2025.1.0). | Img+Vid+GIF | Both | 2020.0.0; app 2023.0.0 | [SLACK] [W20] [W23] [M24] [M25] |
| **Screencast** (formerly Screencast.com) | Upload with title/description prompt and a character-limit warning; link copied. Mac optimizes video before upload (cancellable). Free tier: unlimited images and 25 videos, 12-month retention; Pro: unlimited videos and AI captions, titles and chapters. Viewers need no account; optional access phrase; timestamped comments. | Img+Vid | Both | Renamed 2023.0.0 | [SHARE] [LINK] [SCINFO] [M24] [W25] |
| **Google Drive** | Google sign-in with limited file scope. Uploads only to a "TechSmith" folder (Win) or "TechSmith Snagit" folder (Mac); no folder choice since 2024 (Google policy). File-name prompt. Permission customization (M2026.3.0; Win guidance W2026.0.0). | Img+Vid | Both | — | [SHARE] [GDRIVE] [W24] [M26] [W26] |
| **Dropbox** | Upload and copy link. Mac: "ask for file name and location each time", rename/overwrite, cancel (M2024.2.3). | Img+Vid | Both | — | [SHARE] [M24] |
| **Box** | Upload and copy link; browser-based auth (W2025.2.0). Default on Mac since M2026.3.0 (earlier added manually). | Img+Vid | Both | 2020.0.0 | [SHARE] [W20] [W25] [M26] |
| **YouTube** | Upload video, copy link, provide embed code. Sign-in must grant YouTube management. EULA covers YouTube API (2024.3.0 / M2024.4.0). | Vid | Both | — | [SHARE] [W24] [W19] |
| **Camtasia Editor** | Send as source media; multi-file (2025.4.0). Step Captures become a timeline (needs Camtasia 2026.2.0+). "Edit in Snagit" round-trip from the Camtasia media bin (Camtasia 2025.2.0+). Editor toolbar button sends video to Camtasia (2024.2.2 / M2024.4.0). Watermark without a Camtasia subscription. | Img+Vid | Both | — | [SHARE] [W25] [W26] [W24] [M24] [STEPOUT] |
| **Camtasia Online** | Send images (2025.4.0), videos (W2026.2.0 / M2026.1.0) and Step Captures (2026.3.0). Images sent in original .snagx form. Video can record directly to Camtasia Online via Open In (2026.4.0). | Img+Vid | Both | 2025.4.0 | [SHARE] [W26] [M26] [WN] [WN26] |
| **Evernote** | Create a note. | Img | Win | W2020.1.0 | [SHARE] [W20] |
| **Pages / Keynote / Numbers** | Insert into documents, presentations or sheets. Pages has new/current-document options (M2026.4.0). Step Captures to Pages (M2025.2.0) and Keynote (M2026.0.0) are editable. | Img+Vid | Mac | M2020.1.0 | [SHARE] [M20] [M25] [M26] |
| **Misc Apple apps** | AirDrop, Mail, Messages, Notes, Photos, Freeform, Journal, Reminders (via macOS share). | Img+Vid | Mac | — | [SHARE] |
| **Print / PDF (Step Capture)** | File > Print, or Save As PDF. | Img | Both | — | [STEPOUT] |
| **Markdown / HTML export (Step Capture)** | File > Export produces a folder with the .md or .html file plus images. | — | Mac | MD M2025.3.0; HTML M2026.3.0 | [STEPOUT] [M25] [M26] |
| **Removed destinations** | X/Twitter (W2026.4.0; was Win only); Knowmia (2024–2025, service retired; formerly TechSmith Relay); Panopto (W2024.3.3, M2026.0.0); older ones: Facebook, Flickr, Vimeo (Mac), MediaCore, MindManager, Visual Studio Team System, Snagit Notes, Movable Type, OneNote 2007. | — | — | — | [W26] [W25] [W24] [M24] [M26] [HM4] [PLUG] |

---

## 4. Presets, themes, Quick Styles and Assets

| Feature | Behavior | Platform | Version | Sources |
|---|---|---|---|---|
| **Capture presets** | Save capture type, effects, share destination and file format. Create "from current settings" or via New Preset; edit, rename, delete. | Both | — | [PRESET] [SPEED] [CUST] |
| **Preset hotkeys** | Add Hotkey (Win) / + Shortcut (Mac). | Both | — | [PRESET] [SPEED] |
| **Preset reorder** | Drag to reorder. | Both | Win (2022 help); M2025.4.0 | [H22] [M25] |
| **Preset export/import** | Export a single preset to .snagpresets; File > Import Presets (multi-select). Admins can bundle presets in deployments. | Both (?) | ≤2020 | [PRESET] [H20] [H23] [DEPW] |
| **Open In (per capture / preset)** | Replaces "Preview in Editor": Snagit Editor, None/Library only, Camtasia Editor, Camtasia Online (video). | Both | M2025.3.0 / W2026.1.0; Online 2026.4.0 | [W26] [M25] [WN26] |
| **Built-in special presets** | Step Capture preset; video preset that opens in Camtasia; Slack-sharing preset. | Both | 2025.0.0 / 2026.2–2026.3 | [W25] [W26] [M26] [W23] |
| **Legacy profiles** | Snagit 12 and earlier: .snagprof profiles, Organize Profiles, Export All (optionally with hotkeys and passwords). | Win (legacy) | ≤12 | [H12] |
| **Themes** | Named set of colors (4–8 recommended; preset, palette, HEX/RGB or eyedropper), one font and a shadow on/off switch. Quick Styles are generated from the theme. Create, edit (overwrites generated styles), export to .snagtheme, import (needs Win 13.1+ / Mac 4.1+). Themes transfer across platforms. Default "Starter" theme since 2024.0.0. "Themes 2.0" (2021.0.0) added organization-shared fonts. | Both | — | [THEME] [W21] [M21] [W24] |
| **Quick Styles** | Save tool properties with +, drag to reorder, right-click to delete, reset per tool (M2020.1.4). | Both | — | [QS] [SPEED] [M20] |
| **Favorites (tools/styles)** | Star a Quick Style or Quick Effect; it appears in the Favorites tool. Effects favorites are Mac only. | Both | M2019.0.0 | [QS] [M19] [M23] |
| **Quick Effects** | Save customized effects ("Add to Quick Effects" on Win; + on Mac). | Both | — | [QS] |
| **TechSmith Assets** | Stamps, photos, themes, templates and AI images (Premium). Starter tier free; Premium is an annual subscription. Win: Assets button opens the website and uses "Send to Snagit". Mac: in-app browse, search and download (M2025.4.0). Items land in the Stamp tool, tray/Library, Quick Styles panel or template picker. | Both | Win Assets button 2024.3.0 | [ASSETS] [W24] [M25] [M26] |
| **Toolbar customization** | More > Customize Toolbar: add tools, effects, menu items or share destinations, add spacers, Icon Only labels, Restore Defaults. | Both | — | [TOOLBAR] [SPEED] |

---

## 5. Preferences / Settings

### 5.1 Windows: Capture Preferences (File > Capture Preferences)

| Tab | Settings | Sources |
|---|---|---|
| General | Run at Windows startup; interface = Capture Window or Capture Widget (edge tab); color theme Light/Dark/System; update speed Standard/Delayed (2026.4.0, admin-lockable); automatically check for updates. | [PREFW] [W26] |
| Capture | Hide Snagit during capture; open screenshots from system tools (Snipping Tool, Snip & Sketch) in the Editor (integration since W2019.1.0); magnifier; simplified crosshairs; require Start for scrolling capture; muted-mic alert; HDR support (W2024.2.2); auto-start recording; countdown; Ready to Record window; minimize controls; scale to 1080p; webcam; software encoding; video quality; Printer Capture Settings. | [PREFW] [W19] [W24] |
| Hotkeys | Change hotkeys (global capture, repeat last capture, video); allow Snagit to override other apps' hotkeys (Print Screen on Windows 11). | [PREFW] [GHK] [W23] |

### 5.2 Windows: Editor Preferences (Edit > Editor Preferences)

| Tab | Settings | Sources |
|---|---|---|
| General | Color theme Light/Dark/System (System added W2026.2.0); language EN/FR/DE/JA/PT/ES; show Capture Widget; onboarding; tooltip detail (Off / Text / Video+text); Data & Privacy (personalized usage data / anonymous / none); help and notification level. | [PREFW] [W26] |
| Editor | Workspace color; recent-file thumbnail size; auto-expand canvas; Esc closes Editor; cursor Quick Style popup; toolbar labels; spell check and language. | [PREFW] |
| Share | Enable/disable destinations plus their options; exclude capture source from names. | [PREFW] |
| Library | Tray size 1–200; autosave images; autosave videos; prompt before delete; clear unpinned tray items on close; Library location; convert prompt; Import Backup. | [PREFW] |
| Advanced | Keep Editor running in background; delete cropped content; auto-flatten pasted images; drag-and-drop file type (W2024.1.2); Mobile Sharing (keep original dimensions, connection names); Reset to Default Settings (W2024.0.1). | [PREFW] [W24] |
| Experiments | Toggle each experiment; details dropdown; Send Feedback button; anonymous data collected. | [PREFW] [W25] [W26] |

### 5.3 Mac: Snagit > Settings

| Area | Settings | Sources |
|---|---|---|
| General | Keep running in the menu bar; "Snagit Shows" (capture UI style); Editor theme Light/Dark/System (System since M2020.1.4); tooltip detail; tray count 1–200; anonymous usage info; notifications; crash reports; auto-check updates (with frequency); auto download and install; Standard/Delayed updates (M2026.4.0); reset "Don't Ask Me" warnings. | [PREFM] [M20] [M26] |
| Capture | Hide Snagit while capturing; hide desktop icons (M2024.3.0); minimized recording controls; open system screenshots/videos in Snagit (with Keep original file); open AirDrop items in Snagit (M2024.3.0); scrolling Start button; muted-mic alert; Fix Capture Permissions; frame rate; downsample Retina video; countdown; auto-start; Ready to Record; combine audio tracks; system-audio component; H.265 (M2025.2.0). | [PREFM] [M24] [M25] |
| Share | Destination list, order, +/–, Share Names, per-destination options and shortcuts; exclude capture source. | [PREFM] [SHARE] |
| Keyboard | Change global and other shortcuts. | [PREFM] [GHK] |
| Advanced/Editor | Scale down retina on share; keep a duplicate editable copy on save; auto-expand canvas; quick edit toolbar; monochrome menu-bar icon; defaults for New Image; Library Location; default sharing format; Mobile Sharing on/off; turn off auto-select of annotations after drawing (M2026.1.0). | [PREFM] [M26] |
| Experiments | Toggle experiments; hidden when analytics are disabled by deployment (M2024.2.0). | [PREFM] [RNM24] |

### 5.4 Cross-cutting app-level settings

| Feature | Notes | Platform | Sources |
|---|---|---|---|
| **Language** | Single installer covers all languages; choose at install or later (W2019.0.0). ES/PT/JA added 2021.0.0. Mac language selection method not documented (?). | Win (Mac ?) | [W19] [W21] [M21] [PREFW] |
| **Updates** | In-app updater (BITS on Win, W2023.2.1); proxy list for the updater; Check for Updates also in the Account dropdown (W2024.2.0); Standard/Delayed rings (2026.4.0). | Both | [W23] [W24] [W26] [M26] |
| **Telemetry and notifications** | Usage-data choices control in-app tips and news; crash reports (Mac); in-app message about data collection (W2024.1.0 / M2024.4.0). | Both | [PREFW] [PREFM] [W24] [M24] |
| **Reset to defaults** | Win button on the Advanced tab; Mac "Reset Don't Ask Me". | Both | [PREFW] [PREFM] |
| **Hotkeys (app level)** | Global capture: Print Screen (Win) / Control+Shift+C (Mac). Copy All, Share Link, Save/Save As/Save All, New from Clipboard, Duplicate, Delete, Pin, tray navigation, Share History (Mac), View Library (Mac). | Both | [HOTK] [HOTKM] [GHK] |
| **Accessibility** | Full keyboard navigation with focus indicator; screen-reader labels; dark theme for contrast; OS DPI scaling; ongoing screen-reader fixes in share dialogs and preferences (2026). | Both | [ACCESS] [W26] |
| **Help / feedback** | "?" icon with Tech Support, tutorials and feedback (W2026.4.0); Learning Center and guided tours (2024.0); welcome survey. | Both | [W26] [NEO2400] [DEPW] |

---

## 6. Copy, drag-and-drop and clipboard integration

| Feature | Behavior | Platform | Version | Sources |
|---|---|---|---|---|
| **Copy All** | Copy the whole image or video, annotations included (Ctrl+Shift+C / Option+Cmd+C). Better transparency handling (W2022.0.1). Copying a video copies the media, not a file path (Mac fix M2024.4.3). | Both | — | [SHARE] [HOTK] [W22] [M24] |
| **Copy selection** | The Selection tool plus Ctrl/Cmd+C copies just a region. | Both | — | [H18W] [HM4] |
| **New from Clipboard** | Ctrl+Shift+N / Shift+Cmd+N creates a new image from the clipboard. | Both | — | [HOTK] |
| **Drag out of tray/Library** | Drop files into other apps such as Teams or Office. Default type for .snagx drags is set on Win Advanced tab; Mac uses "Default sharing format" and Shift to skip retina scaling. | Both | W2024.1.2 | [W24] [PREFM] [TEAMS] |
| **Drag in** | Drop browser images or files onto the tray or canvas; emoji from the macOS Character Viewer (M2025.3.0). | Both | — | [M23] [M25] |

---

## 7. Experiments program (Experiments tab)

General: on/off toggles with details, Send Feedback, anonymous usage collection; features may change or disappear and carry no tech support. Hidden or blocked when analytics or experiments are disabled by deployment (Mac `DisableExperiments`; Win Deployment Tool option). Sources: [PREFW] [PREFM] [RNM24] [DEPW] [DEPM]

| Experiment (as described) | Platform | Version | Later status | Sources |
|---|---|---|---|---|
| Smart text selection ("Smart Highlight") for Highlight/Blur/Spotlight | Mac | M2024.1.0 | — | [RNM24] [M24] [SOFTP] |
| Oval and Pen Screen Draw tools for video | Mac | M2024.1.0 | — | [RNM24] |
| Corner rounding for canvas and floating images ("Canvas Corner Radius") | Mac | M2024.1.0 | Refined 2024.2.5–2024.3.2 | [M24] [SOFTP] |
| Pixel distance measurement ("Live Measure") | Mac | M2024.1.0 | — | [M24] |
| 60 fps recording option | Mac | M2024.1.0 | — | [M24] |
| Capture Hotkeys in Video (shows keystrokes in recordings) | Mac | M2024.2.0 | — | [RNM24] [M24] |
| Auto-detect and blur sensitive info (phone, email, SSN, cards, faces) | Mac | M2024.2.3 | Shipped as Smart Redact 2025.0.0 | [M24] [M25] [W25] |
| Single-window recording that follows window moves and resizes | Mac | M2025.0.0 | — | [M25] [SOFTP] |
| Fullscreen capture with each window as a separate editable layer | Mac | M2025.0.0 | — | [M25] |
| Custom QR code creation; QR code for shared captures; QR scanning during capture | Mac | M2025.4.0 | — | [M25] |
| HDR Capture Fidelity | Win | W2025.4.0 | — | [W25] [NEO2540] [FILI] |
| Improved OCR for finding and editing text in images | Win | W2026.0.0 | — | [W26] [WN26] |
| Quick Image Annotation (capture, annotate and copy to clipboard in one step) | Win | W2026.1.0 | — | [W26] |
| Optimized OCR (better Smart Redact accuracy) | Win | W2026.3.0 | — | [W26] [NEO2630] |
| Screen Draw tools available any time (useful during screen sharing) | Mac | M2026.3.0 | — | [M26] |

---

## 8. Integration points

| Integration | Behavior | Platform | Version | Sources |
|---|---|---|---|---|
| **TechSmith Fuse (mobile companion)** | iOS 13.6+ / Android 7+ app. Editor File > Connect Mobile Device shows a QR code; same Wi-Fi network; Snagit opens a local port (8303) while Mobile Sharing is on; media arrives in the tray. Encrypted with TLS (2021–2022). Admins can disable it (M2026.1.0). Still documented in 2025–2026 tutorials. | Both | — | [MOBILE] [FUSE] [DEPW] [DEPM] [M21] [W22] |
| **TechSmith Capture (old mobile app)** | Retired; no longer supported. | — | — | [TCEOS] |
| **AirDrop to Snagit** | Received items open in the Editor (Capture setting). | Mac | M2024.3.0 | [MOBILE] [M24] |
| **Continuity Camera import** | Image > Import from iPhone/iPad. | Mac | M2025.2.0 | [MOBILE] [M25] |
| **System screenshot capture-in** | Win: open Snipping Tool / Snip & Sketch screenshots in the Editor. Mac: open system screenshots and videos in Snagit (move or keep original). | Both | W2019.1.0 | [PREFW] [PREFM] [W19] |
| **Snagit Printer** | Virtual printer that sends any app's print job to the Editor; configured in Printer Capture Settings; optional install; not on Windows ARM. | Win | Long-standing | [PREFW] [H20] [H22] [DEPW] [COM25] |
| **Windows Explorer extension** | Context-menu batch export integration (installable component). | Win | — | [DEPW] |
| **COM Server API** | Out-of-process COM server (`Snagit.ImageCapture`, `IImageCapture`/`2`, `IVideoCapture`–`4`). Inputs: desktop, window, region, file, clipboard, menu, object, freehand, extended window, All-in-One. Image outputs: none, printer, file, clipboard, email, FTP. Video outputs: preview, file, FTP. Image filters; automatic file naming (prompt / fixed / prefix + counter); capture and recorder events; SNAGX support (22.0.1). Registered with `SnagitCapture.exe /register`. Settings are not persisted to the app. | Win | 8.1+; documented through 2025 | [COM25] [COM12] [COM11] [W22] [W20] |
| **AppleScript / Shortcuts** | No public scripting dictionary found. Only related item: Align menu items renamed so they can be bound as system keyboard shortcuts (M2025.4.0). Mac admin settings use `defaults write` plist keys. | Mac | — (?) | [M25] [DEPM] |
| **Office add-ins** | Current versions use share destinations, not an Office add-in. A legacy "Snagit 10 Combined Add-In" existed (?). | Win (legacy) | ≤10 (?) | [OFFICE] [SHARE] |
| **Browser extension** | "Snagit for Chrome" was discontinued along with other free TechSmith products (exact year not confirmed (?)). No current browser extension. Mac can capture a page by dragging a URL onto the Dock icon (legacy). | — | — | [FT4T] [HM4] [H21] |
| **Slack app** | Install from the Slack App Directory for inline previews. | Both | 2023.0.0 | [SLACK] |
| **Camtasia round-trip** | Edit in Snagit from Camtasia; Edit in Camtasia button (removed if the destination is disabled, M2026.3.2); crash recovery for videos set to open in Camtasia (M2026.1.0). | Both | 2024–2026 | [W25] [M26] [WN] |
| **SNAGX on iOS** | Not openable on iOS; export to PNG first. | — | — | [IOS] |

---

## 9. Licensing, accounts and admin controls (user-visible only)

| Feature | Behavior | Platform | Version | Sources |
|---|---|---|---|---|
| **Subscription model** | Annual subscriptions (individual about $39/yr; included with Camtasia plans). Business seats are transferable; individual seats are not. Included: Screencast with 25 videos, starter Assets, Camtasia Online. Subscriptions offered W2024.1.1 / M2024.2.1; 2025 subscription-only. Store page lists a money-back guarantee and no trial. | Both | 2024–2025 | [PRICE] [W24] [M24] [W25] [M25] |
| **Sign-in unlocks** | TechSmith account sign-in (no key needed for subscriptions). Software keys and offline activation still supported (2025.1.0). | Both | — | [PRICE] [W25] [M25] |
| **Account dropdown** | Profile photo and name, license and expiry info, notifications, update check, sign out; opens automatically for subscription alerts. | Both | 2024.2.0+ | [W24] [M24] [W25] |
| **Expired/trial states** | Expired-subscription dialog (sign in / switch account), offline expiry message, trial messaging. | Both | 2025–2026 | [W26] [M26] |
| **Camtasia activation** | Camtasia Editor subscribers can activate Snagit. | Both | W2026.2.2 / M2026.2.0 | [W26] [M26] |
| **Product naming** | Windows: "Snagit" without a year from 2026.0.0. Mac app renamed "Camtasia Snagit" in 2026.0.0. | Both | 2026.0.0 | [WN26] [M26] |
| **Platform requirements** | macOS 14+, Windows 10/11 64-bit. | Both | 2025–2026 | [PRICE] [COM25] |
| **Admin deployment controls** | Win Deployment Tool (MST), with user-visible effects: disable updates, welcome survey/tours, Share Manager, individual destinations, Share Link target, sign-in, usage data, help level, Experiments, Assets links, video or image capture, printer capture, product key display. Language, Library location and lock, bundled presets and themes, registry defaults. Mac equivalents via plist keys: RestrictedOutputs, ConfigureShareLink, DisableProductLogin, DisableTracking, DisableExperiments, DisableCheckForUpdates, DisableUpdateRing, DisableFuseSharing, DisableFeedbackSurvey, DisableVideoCapture, DisableAssetService, HideRegistrationKey. | Both | — | [DEPW] [DEPM] |

---

## 10. Gaps and uncertainties for the parity team
- **Share History:** no full public spec exists. The columns and actions above come from release notes and old help files (?).
- **FTP:** exact UI fields (host, path, credentials, SFTP support) are not publicly documented beyond passive mode, a custom port (Mac) and automatic naming (COM level).
- **Mac language switching** and **Mac preset export/import** UI are unconfirmed (?).
- **Windows Capture Info / "Details"** field list is not documented beyond time, app, URL and OS.
- **Windows Experiments before 2025.4.0:** no public list found. The Mac 2024 experiments may not all have shipped on Windows.

---

## Deduplicated URL list (all consulted)

**Tutorials (techsmith.com)**
- [LIB] https://www.techsmith.com/learn/tutorials/snagit/snagit-library/
- [LIBLOC] https://www.techsmith.com/learn/tutorials/snagit/snagit-library-location/
- [MANLIB] https://www.techsmith.com/learn/tutorials/snagit/manage-library/
- [CONV] https://techsmith.com/blog/tutorials/snagit/convert-previous-captures
- [TRAY] https://www.techsmith.com/learn/tutorials/snagit/recent-captures-tray/
- [SAVE] https://www.techsmith.com/learn/tutorials/snagit/file-save-tips/
- [MOBILE] https://techsmith.com/learn/tutorials/snagit/import-media-from-mobile
- [FUSE] https://www.techsmith.com/learn/tutorials/mobile-apps/fuse-mobile/
- [SHARE] https://www.techsmith.com/learn/tutorials/snagit/share/
- [SHARE-OLD] https://techsmith.com/tutorial-snagit-share.html
- [LINK] https://www.techsmith.com/learn/tutorials/snagit/share-link/
- [SLACK] https://www.techsmith.com/learn/tutorials/snagit/snagit-slack-app/
- [TEAMS] https://www.techsmith.com/learn/tutorials/snagit/share-to-microsoft-teams/
- [STEPOUT] https://www.techsmith.com/learn/tutorials/snagit/step-capture-outputs/
- [BATCH] https://www.techsmith.com/learn/tutorials/snagit/batch-convert-images/
- [PREFW] https://www.techsmith.com/learn/tutorials/snagit/preferences/
- [PREFM] https://www.techsmith.com/blog/tutorials/snagit/preferences-mac/
- [HOTK] https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/
- [HOTKM] https://www.techsmith.com/learn/wp-content/uploads/2024/07/Snagit-Shortcuts-Guide-Printable-Mac.pdf
- [GHK] https://www.techsmith.com/learn/tutorials/snagit/change-global-hotkey/
- [PRESET] https://www.techsmith.com/tutorials/snagit/presets/
- [THEME] https://www.techsmith.com/learn/tutorials/snagit/snagit-themes/
- [QS] https://www.techsmith.com/learn/tutorials/snagit/quick-styles-and-favorites/
- [CUST] https://techsmith.com/learn/tutorials/snagit/customize-snagit
- [TOOLBAR] https://www.techsmith.com/learn/tutorials/snagit/customize-toolbar/
- [SPEED] https://techsmith.com/learn/tutorials/snagit/speed-up-snagit-workflow
- [ASSETS] https://www.techsmith.com/learn/tutorials/snagit/assets/
- [ACCESS] https://www.techsmith.com/learn/tutorials/academy/guide-for-using-snagit-accessibly/
- [TRANS] https://www.techsmith.com/blog/tutorials/snagit/translate-text/
- [GTK] https://www.techsmith.com/blog/tutorials/snagit/get-to-know-snagit/
- [EDIT] https://www.techsmith.com/learn/tutorials/snagit/how-to-edit-a-screenshot/
- https://techsmith.com/learn/tutorials/snagit/whats-new-2022
- [PREV] https://www.techsmith.com/learn/tutorials/snagit/snagit-previous-version-help/
- https://www.techsmith.com/learn/tutorials/section/snagit/find-and-manage-captures
- https://www.techsmith.com/learn/tutorials/section/snagit/customize-snagit/

**What's new and update messages**
- [WN] https://www.techsmith.com/snagit/whats-new/ (also /page/2/, /page/5/)
- https://techsmith.com/snagit/whats-new/access-snagit-features-faster-with-the-new-right-click-menu
- [WN26] https://download.techsmith.com/update/snagit/enu/26_updatemessage.html
- [RNM24] https://download.techsmith.com/update/snagitmac/enu/2420/rnotes.html

**Version histories (support.techsmith.com)**
- [WIDX] https://techsmith.com/snagit-version-history.html, which redirects to https://support.techsmith.com/hc/en-us/sections/39094961933453
- [W26] https://support.techsmith.com/hc/en-us/articles/42674936732685-Snagit-Windows-2026-Version-History
- [W25] https://support.techsmith.com/hc/en-us/articles/39102293598349-Snagit-Windows-2025-Version-History
- [W24] https://support.techsmith.com/hc/en-us/articles/39102250364429-Snagit-Windows-2024-Version-History
- [W23] https://support.techsmith.com/hc/en-us/articles/39102261751949-Snagit-Windows-2023-Version-History
- [W22] https://support.techsmith.com/hc/en-us/articles/43410048034061-Snagit-Windows-2022-Version-History
- [W21] https://support.techsmith.com/hc/en-us/articles/43410747342605-Snagit-Windows-2021-Version-History
- [W20] https://support.techsmith.com/hc/en-us/articles/43410801744525-Snagit-Windows-2020-Version-History
- [W19] https://support.techsmith.com/hc/en-us/articles/43414049971213-Snagit-Windows-2019-Version-History
- [MIDX] https://support.techsmith.com/hc/en-us/articles/115006443147
- [M26] https://support.techsmith.com/hc/en-us/articles/41975263481613-Snagit-Mac-2026-Version-History
- [M25] https://support.techsmith.com/hc/en-us/articles/37938520706957-Snagit-2025-Version-History
- [M24] https://support.techsmith.com/hc/en-us/articles/37938485483021-Snagit-Mac-2024-Version-History
- [M23] https://support.techsmith.com/hc/en-us/articles/37938496753549-Snagit-Mac-2023-Version-History
- [M22] https://support.techsmith.com/hc/en-us/articles/43415422714125-Snagit-Mac-2022-Version-History
- [M21] https://support.techsmith.com/hc/en-us/articles/43415439595533-Snagit-Mac-2021-Version-History
- [M20] https://support.techsmith.com/hc/en-us/articles/43415462314125-Snagit-Mac-2020-Version-History
- [M19] https://support.techsmith.com/hc/en-us/articles/43415505223437-Snagit-Mac-2019-Version-History

**Support articles**
- [PLUG] https://support.techsmith.com/hc/en-us/articles/115002022732-Download-Snagit-Share-Destinations
- [PERM] https://support.techsmith.com/hc/en-us/articles/19598294359565-Snagit-Share-Destination-Permissions-FAQ
- [MISSING] https://support.techsmith.com/hc/en-us/articles/360059898111-Snagit-Share-Destinations-Missing
- [SECT1] https://support.techsmith.com/hc/en-us/sections/200514888-Share
- [SECT2] https://support.techsmith.com/hc/en-us/sections/200514928-Share
- [GDRIVE] https://support.techsmith.com/hc/en-us/articles/40294019257741-Cannot-Select-Destination-Folder-in-Google-Drive
- [EMAIL] https://support.techsmith.com/hc/en-us/articles/28032096680205-Share-to-Email-failed-error-message
- [ONENOTE] https://support.techsmith.com/hc/en-us/articles/16316597314061-Cannot-Share-to-OneNote
- [FTP] https://support.techsmith.com/hc/en-us/articles/203732348-Snagit-Share-to-FTP-Not-Working
- [OFFICE] https://support.techsmith.com/hc/en-us/articles/360029399471-Share-to-Word-PowerPoint-Excel-or-OneNote-from-Snagit-Video
- [IOS] https://support.techsmith.com/hc/en-us/articles/33753217605005-Open-a-SNAGX-file-on-iOS
- [EXPORTLIB] https://support.techsmith.com/hc/en-us/articles/27142747345805-How-Do-I-Export-My-Snagit-Library-if-I-No-Longer-Plan-to-Use-Snagit
- [SEARCH] https://support.techsmith.com/hc/en-us/articles/4411564911501-Snagit-Search-Not-Working
- [NODISP] https://support.techsmith.com/hc/en-us/articles/4420370583309-Snagit-2022-Library-does-not-display-files
- [EMPTY] https://support.techsmith.com/hc/en-us/articles/360061181012-Snagit-Library-is-Empty-After-Update
- [BACKUP] https://support.techsmith.com/hc/en-us/articles/203731768
- [SAVED21] https://support.techsmith.com/hc/en-us/articles/203732798
- [TCEOS] https://support.techsmith.com/hc/en-us/articles/35674291844749-TechSmith-Capture-End-of-Support
- [DEPM] https://support.techsmith.com/hc/en-us/articles/115007344888-Enterprise-Install-Guidelines-for-Snagit-on-MacOS

**Guides and PDFs**
- [DEPW] https://www.techsmith.com/wp-content/uploads/2026/02/Snagit-Deployment-Tool-Guide.pdf
- [COM25] https://assets.techsmith.com/Docs/Snagit-2025-COM-Server-Guide.pdf
- [COM12] https://assets.techsmith.com/Docs/pdf-snagit/COM_Documentation.pdf
- [COM11] https://assets.techsmith.com/Docs/pdf-snagit/Snagit_COM_Server_Documentation.pdf
- [H24] https://www.techsmith.com/learn/wp-content/uploads/2025/01/Snagit-Help-2024-EN.pdf
- [H23] https://www.techsmith.com/learn/wp-content/uploads/2023/09/Snagit-Help-2023-EN.pdf
- [H22] https://www.techsmith.com/learn/wp-content/uploads/2022/10/Snagit-Help-2022.pdf
- [H21] https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-Help-2021.4.pdf
- [H20] https://gulfcoast.edu/administration-departments/information-technology-services/staff/documents/snagit-complete-help-guide-2020.pdf
- [H18W] https://www.techsmith.es/blog/wp-content/uploads/2021/07/Snagit-Windows-Help-2018.1.pdf
- [H18M] https://www.techsmith.es/blog/wp-content/uploads/2021/07/Snagit-Mac-Help-2018.1.pdf
- [HM4] https://www.techsmith.es/blog/wp-content/uploads/2021/07/Snagit-Mac-Help-PDF.pdf
- [H12] https://assets.techsmith.com/Docs/pdf-snagit/Snagit_12.4.1_Help.pdf
- [H10] https://assets.techsmith.com/docs/pdf-snagit/Snagit-10-Getting-Started.pdf
- [SCINFO] https://www.techsmith.com/wp-content/uploads/2026/04/TechSmith_Snagit_Screencast_Info_Sheet.pdf
- [PRICE] https://Techsmith.com/snagit-pricing.html

**Third-party reviews and changelog mirrors**
- [NEO2630] https://www.neowin.net/amp/snagit-202630/
- [NEO2540] https://www.neowin.net/amp/snagit-202540/
- [NEO2400] https://www.neowin.net/software/snagit-20240/
- [FILI] https://thefilibusterblog.com/snagit-2025-enhance-your-screen-capture-experience-with-version-2025-4-0/
- [BETA] https://betanews.com/article/snagit-2024/
- [SOFTP] https://mac.softpedia.com/progChangelog/Snagit-Changelog-68434.html
- [FT4T] https://freetech4teach.teachermade.com/?p=13127

**Fetched but not useful (unrelated content):** https://apps.microsoft.com/detail/9n9j36x5f9b8?hl=en-US

**Attempted but failed to load:** https://www.neowin.net/software/snagit-202540/, https://www.neowin.net/software/snagit-20260/
