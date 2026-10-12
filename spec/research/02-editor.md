# Snagit Editor: clean-room feature inventory (public sources only)

**How this was built.** I used only public web sources: TechSmith Learn tutorials, TechSmith support articles, the official per-year version histories for Windows (2019–2026) and Mac (2019–2026), official product and feature pages, the official help PDFs for older versions, and a few press articles. I did not download, install or inspect any Snagit binary, installer or `.snagx` file. Everything below is in my own words.

**Search budget.** The shared web-search budget ran out near the end of the session. After that I only fetched URLs I already knew. The items marked (?) and the "Open questions" section are the places a follow-up search would help most.

**Legend**
- **Win** = Windows, **Mac** = macOS, **Both** = both platforms.
- **(?)** = uncertain or inferred; not confirmed by a current official source.
- **"Legacy doc"** = taken from an older official help PDF (Snagit 12, 13 or 2018–2021); it may no longer apply.
- **Version numbering.** From 2024 on, versions are written YYYY.minor.patch (for example 2025.1.0). Older Windows versions were numbered 13 and below; Mac versions were 4.x and below.

---

## 1. Editor shell and UI

| Name | Behavior and options | Platform | Version | Source |
|---|---|---|---|---|
| **Editor tab vs Library tab** | The editor window has a top-level switch between the Editor (preview, edit, share) and the Library (capture history). Since 2025 both are on the main toolbar. | Both | Win 2025.3.0; Mac 2025.2.0 | how-to-edit-a-screenshot; Win/Mac 2025 version history |
| **Assets entry** | Browse, search and download stamps, photos, themes and templates. On Mac this is an in-app Assets tab; on Windows the button opens the Assets website, which uses "Send to Snagit". Premium assets need a subscription. AI-generated images exist only on the Assets website, for premium subscribers. | Both (in-app browsing is Mac only) | Win 2024.3.0; Mac button 2024.3.2, in-app tab 2025.4.0, photos 2026.1.0 | assets; Mac 2024–2026 version history |
| **Capture button** | Opens the Capture window from inside the editor. | Both | — | how-to-edit-a-screenshot |
| **Create menu** | Two entries: Image from Template, and Video from Images. | Both | 2020 | how-to-edit-a-screenshot |
| **Toolbar** | Annotation and editing tools, plus a **More** dropdown for extra tools. Customizable (see §2.1). Tools work on images only. | Both | — | customize-toolbar |
| **Share area** | Three parts: **Copy All** (copies the flattened image), **Share Link**, and a **Share dropdown** for destinations or saving to a file. | Both | — | how-to-edit-a-screenshot |
| **Workspace / Canvas** | The central editing area. Workspace background color is set in preferences (Win). | Both | — | preferences |
| **Recent Captures Tray** | Strip of thumbnails for recent captures and opened files; clicking one loads it on the canvas. Full detail in §1.1. | Both | — | recent-captures-tray |
| **Side panel: Quick Styles, Tool Properties, Effects** | Right-hand sidebar. It can be hidden (View > Side Bar on Win, Sidebar on Mac; Mac shortcut Cmd+5). Mac Cmd+4 shows the Effects/Properties panel. | Both | — | 2024 help PDF; hotkeys |
| **Quick edit toolbar** | A small floating properties toolbar near the selected object (callout, text, shape and others). Can be turned off in settings. | Mac | hide option added Mac 2019.1.1 | preferences-mac; Mac 2019 history |
| **Cursor Quick Style popup** | Shows a preview of the current tool style next to the pointer. | Win | — | preferences |
| **Image dimensions readout** | Image size is shown under the canvas; clicking it opens Resize Image. Mac experiment: corner radius is shown alongside it. | Both (radius readout Mac) | Mac 2024.3.0 (exp.) | resize-rotate-flip-image; Mac 2024 history |
| **Zoom** | Actual Size (Ctrl/Cmd+0), Zoom to Fit (Mac Shift+Cmd+0), Zoom In/Out (Ctrl/Cmd +/−; Mac also Option+scroll). Win has a Zoom Options control above the tray with **Always Shrink to Fit**, which is on by default so large captures open zoomed out. Mac remembers zoom and scroll between launches. | Both | Mac remember: 2021.2.0 | hotkeys; support 235941148; Mac 2021 history |
| **Pixel Grid** | A pixel grid overlay at high zoom. A 2025 fix still mentions its checkmark. The legacy doc says it appears at 600% zoom and above. | Win (?) | still present Win 2025.1.0 | Win 2025 history; Snagit 12.4.1 PDF |
| **Full screen** | Fn+F toggles full screen. | Mac | — | hotkeys |
| **Help / feedback menu** | A "?" or Help menu with support, tutorials and product feedback. An earlier feedback button sat at the top right. | Both | Win 2024.1.0 / 2026.4.0; Mac 2023.2.6 / 2025.4.0 | version histories |
| **Account dropdown** | Shows profile photo and license info, and can check for updates. | Win | 2024.2.0–2024.2.2 | Win 2024 history |
| **Color theme (UI)** | Light, Dark or System. Win added the System option in 2026.2.0. Icons and colors were refreshed in 2026. | Both | refresh: Win 2026.3.0, Mac 2026.0.0 | preferences; whats-new |
| **Enhanced tooltips** | Tooltips with optional short animations. Mac also has tooltips on tool properties and on callout shape styles. | Both | 2022.0.0 (video tooltips); Mac 2024.3.2, 2025.3.0 | preferences; Mac histories |
| **Onboarding pop-ups** | First-use feature introductions, which can be toggled. | Win | — | preferences |
| **Unsaved-changes indicator** | An orange sunburst on a tray item that has unsaved edits since its last save. Mac shows a tooltip on the icon. | Both | Mac tooltip 2026.3.0 | recent-captures-tray; Mac 2026 history |
| **Escape closes Editor** | Optional; off by default. | Win | — | preferences |
| **Learning Center / offline help PDF** | Built-in help, with a PDF offered when offline. | Win | 2025.3.0 | Win 2025 history |
| **Accessibility** | Tab and Shift+Tab focus navigation with a visible focus ring; screen reader labels; DPI scaling. Arrow keys adjust sliders (Win 2026.0.0) and number fields (Win 2024.3.5). Recent releases improved screen reader support in dialogs and panels. | Both | various | accessibility guide; Win 2024–2026 history |

### 1.1 Recent Captures Tray (editor-facing behavior)
- Show or hide with Show/Hide Recent (Win), Recent (Mac), or Option+Cmd+T (Mac).
- Thumbnail size Small, Medium or Large: Win only.
- Number of items kept: 1–200, set in preferences (Win 2024.0.3; Mac 2022.2.0).
- **Multi-select** with Ctrl+click or Cmd+click. With several images selected, the context menu offers **Create Image from Template** (or **Combine in Template**) and **Create Video from Images**.
- **Pin / unpin.** Pinned items anchor to the left; you can pin several and drag to reorder. Win 2024.0.0; Mac 2026.1.0. Shortcut Ctrl+Shift+P (Win) or Control+Shift+P (Mac).
- **Reorder** pinned and unpinned items by dragging.
- **Close** removes an item from the tray without deleting the file.
  - Mac: "Close Other Captures".
  - Win: a preference removes unpinned items when the editor closes.
- Other context menu items:
  - Details (Win) or Get Info (Mac).
  - Delete (Win) or Move to Trash (Mac): this permanently removes the file.
  - Rename: F2 on Win; also File > Rename.
  - Duplicate: Ctrl+Shift+D or Shift+Cmd+D. Mac 2026.3.0 can duplicate several items at once.
- **Next / previous capture:** arrow keys (Win), Cmd+[ and Cmd+] (Mac).
- **Drag a tray item onto the canvas** to add it as a floating image. This is how images are combined.
- Saving to JPG or PNG leaves both the `.snagx` and the exported file in the tray.
- Sources: recent-captures-tray; hotkeys; preferences; whats-new; Win/Mac version histories.

---

## 2. Tool system (cross-cutting)

### 2.1 Customize Toolbar — Both
- Open it from **More > Customize Toolbar**. In the dialog you can:
  - drag items onto the toolbar to add them;
  - drag them along the toolbar to reorder;
  - drag them off to remove them;
  - insert **Space** separators to group items;
  - click Done to finish.
- **Restore Defaults:** a button on Win; on Mac you drag the default set onto the toolbar.
- **Icon Only:** right-click the toolbar (Win); there is also a "Show Toolbar Labels" preference.
- Non-tool items that can be added:
  - Save, Duplicate, New from Clipboard;
  - share destinations (Word, Teams, Google Drive and others);
  - individual effects (Border, Filters);
  - Auto Trim (Win);
  - Revert to Original (Mac 2020.1.3);
  - Send video to Camtasia.
- **More dropdown behavior:** a tool you pick from More temporarily takes a toolbar slot until you pick a different More tool. Pin a tool by dragging it to a fixed position.
- **Default toolbar** (2021 help PDF): Favorites, Arrow, Text, Callout, Shape, Stamp, Fill, Move, Selection.
- **Tools under More:** Blur, Crop, Cut Out, Eraser, Grab Text, Highlighter, Line (Win), Magic Wand, Magnify, Pen, Simplify, Step, Spotlight (Mac).
- Toolbar layout and order persist across restarts (Win 2019.1.1).
- Sources: customize-toolbar; speed-up-snagit-workflow; 2021 help PDF; Win/Mac histories.

### 2.2 Quick Styles — Both
- Each tool ships with preset styles.
- Editing properties creates a temporary style at the end of the list; click **+** to save it.
- Drag to reorder styles; right-click > Delete Quick Style to remove one.
- **Reset Quick Styles** restores a tool's defaults (Mac 2020.1.4; Win fix 2024.0.3).
- Quick Styles are the way to set a tool's default properties.
- Quick Styles are generated from the active **Theme** (§2.4).
- Applying a Quick Style to several selected objects at once is supported (Win fixes in 2023).
- Blur Quick Styles apply only Type and Intensity (Win 2025.1.0).
- Sources: quick-styles-and-favorites; support 45790491793165; Win/Mac histories.

### 2.3 Favorites — Both
- Clicking the star on a Quick Style adds it to the **Favorites** tool (default toolbar, first slot; Mac shortcut Control+R). The star turns yellow; click again to remove.
- Favorites mixes styles from different tools for one-click access.
- Quick Effects can be favorited on Mac only (Mac 2022.2.2 / 2023.0.1).
- Cut Out Quick Styles can be favorited (Mac 2025.2.0).
- Favorites are also the drawing tools offered during Video from Images.
- Introduced in 2019.0.0.
- Sources: quick-styles-and-favorites; speed-up-snagit-workflow; Mac/Win 2019 histories.

### 2.4 Themes — Both
- A theme is a named set of **colors** (4–8 recommended), **one font**, and a **shadow on/off** switch. Snagit generates Quick Styles for most tools from it.
- **Create New / Edit / Export / Import:**
  - Win: from the Theme dropdown, with a gear icon per theme.
  - Mac: from a gear menu.
- **Editing a theme overwrites that theme's Quick Styles.**
- File format: **`.snagtheme`**, used for sharing. Importing needs Win 13.1+ or Mac 4.1+. Mac can also import via File > Import (2021.1.0).
- Themes can be downloaded from Assets.
- Default theme is **"Starter"** since 2024.0.0; an earlier set was named "Industrial" (formerly "Urban").
- "Themes 2.0" (2021.0.0) added font sharing and per-tool shadow toggling.
- In templates, a theme can be applied to existing objects (see §9).
- Edge cases:
  - You cannot delete every color from a theme.
  - Unnamed new themes are kept.
  - Duplicate colors are blocked (Mac fixes).
  - Callout arrows are carried across Mac and Win themes (fix).
- Sources: snagit-themes; modify-template; Win 2021/2024 and Mac 2020–2023 histories.

### 2.5 Color picker — Both
- Ways to choose a color: preset theme colors, palette, Hex or RGB entry (Win has a **Mode** toggle between the two), **Eyedropper** that samples anywhere on screen, and a **Transparent** (checkerboard) choice where it applies.
- Mac has "My Colors" saved colors (synced between Capture and Editor) and "My Gradients".
- Mac color picker was redesigned in 2026.0.0, with better gradient selection and transparency.
- Gradient tooltips show color info (Mac 2026.1.0).
- Win: holding Ctrl switches between the tool and the color picker.
- Sources: arrow, callout, gradients, hotkeys, Mac 2023 and 2026 histories.

### 2.6 Gradients — Mac only
- Available on: canvas color, Shape fill, Callout fill, Fill tool, and the PiP camera border.
- Types:
  - **Linear**, with an angle set by typing degrees or with a dial.
  - **Radial**, from the center.
- Two **color stops**, each with its own alpha. Adding more stops is not documented.
- Saved gradients live under **My Gradients**.
- Added in Mac 2025.0.0.
- Source: gradients; Mac 2025 history.

### 2.7 Shared object-property patterns
- **Shadow (per object):** click one of the direction cells to turn the shadow on; click it again to turn it off. Advanced settings are Color, **Angle (Win only)**, Distance, Opacity and Blur. Mac gained advanced shadow controls in 2022.0.0. Tools with this: Arrow, Line, Text, Callout, Shape, Stamp, Step, Pen.
- **Opacity** slider on most tools.
- **Select All (\<Tool\>)** button in Tool Properties selects every object made by that tool so they can be edited together. Labels include Arrow, Text Object, Callout, Shape, Stamp, Step, Pen Mark, Highlight, Blur, Magnification, Spotlight, Line. Win 2026.1.0; Mac 2025.3.0.
- **Handle conventions:**
  - white = resize;
  - green = rotate;
  - yellow = tail, curve or corner radius depending on the tool;
  - blue = add callout tail;
  - blue handles = crop rectangle.
- **Shift while drawing** constrains: square or circle for shapes; horizontal or vertical for lines, arrows, pen and eraser.
- **Shift while resizing rotated shapes** keeps the aspect ratio (Win 2020.1.4).
- **Command-drag** resizes an object or the canvas from its center (Mac 2025.0.0).
- **Option-drag** draws over existing objects instead of selecting them (Mac).
- Annotations **auto-select after drawing**; this can be turned off (Mac 2026.1.0).
- Font size and arrow width accept decimal values (Win 2023.0.0).
- Sources: arrow, text, shape, stamp, step, pen, highlighter, blur, magnify, spotlight, line tutorials; hotkeys; Mac 2022/2025/2026 and Win 2020/2023 histories.

---

## 3. Annotation tools (vector objects, editable in `.snagx`)

### 3.1 Arrow — Both (Mac shortcut Control+A; on Mac the Arrow and Line tools share this shortcut)
- Drag from tail to head. Shift locks the arrow horizontal or vertical. Delete key removes it.
- **Curved arrows:**
  - Win: a Bezier Curve checkbox, then drag the white handles.
  - Mac: drag the yellow center handle; more handles appear as you edit.
- **Properties:**
  - Color; Shadow (with advanced settings);
  - **End Style** for head and tail, including a "T" end (2022.0.0);
  - **Line Style** (solid, dashed and others);
  - **Width** (slider or pixels);
  - Opacity;
  - **Start/End Size** (resizable arrowheads: Win 2022.1.1, Mac 2022.0.0);
  - Bezier Curve (Win).
- Arrows snap to canvas objects (Mac fix 2025.2.0).
- Some end styles and curved arrows render differently on the other platform (§10).
- Sources: arrow; Win 2022 history; version-compatibility.

### 3.2 Line — Win (the tutorial is titled "Line Tool (Windows)"; on Mac, line drawing appears to live in the Arrow tool (?))
- Drag to draw; Shift constrains. White handles resize.
- Properties: Color, Shadow, End Style, Line Style, Opacity, Bezier Curve. A width property is not shown in the tutorial (?).
- Select All (Line) needs Win 2026.1.0 or later.
- Source: line.

### 3.3 Text — Both (Control+T on Mac)
- Drag to set the box, then type.
- Fixing text already in the screenshot uses **Edit Text** (§5), not this tool.
- **Properties:**
  - **Fill** (letter color) and **Outline** (stroke color around letters);
  - Shadow;
  - Font; Font Size;
  - **Font Style** (Mac: Regular, Italic, Bold, Bold Italic);
  - **Line Width**, the outline width (0 = none);
  - Advanced: Bold and Italic (Win), Underline and Strikethrough (Mac); horizontal and vertical **Alignment**; **Padding** (Win); Opacity;
  - **Line Spacing** (Mac 2024.2.5; Win (?)).
- Text and callouts re-orient after rotation.
- The maximum font size was raised over time; Mac allows more than 300 pt.
- Spell check: Win has a preference with its own language; Mac uses the system "Show Spelling and Grammar". The Win spell checker recognizes product names (2026.0.0).
- **Paste and Match Style** (Mac 2020.2.0).
- Sources: text; preferences; hotkeys; Mac 2019/2020/2022/2024 and Win 2026 histories.

### 3.4 Callout — Both (Control+D on Mac)
- Drag to draw, then type. A callout is a text box with background, border and one or more pointer tails.
- **Handles:**
  - white = resize body;
  - yellow tail handle = reshape the tail; drag it into the body to delete that tail;
  - blue = add another tail (**multiple tails**: Win 2022.0.0);
  - green = rotate;
  - **anchor point** = move the body around the tail;
  - **yellow corner handle = corner radius (Mac)**.
- **Properties:**
  - Fill and Outline colors (gradients on Mac);
  - **Shape** dropdown (shape names are only shown in images; arrow-tail callouts since 2020.0.0; cloud tails; explosion callouts on Mac (?));
  - Shadow;
  - Font, Font Color, Font Size, Font Style (Mac);
  - Outline Size (Win) or Line Width (Mac), where 0 = no outline;
  - **Arrow Style** (line and end style of the tail), Arrow Color, Arrow Size;
  - Advanced: Bold and Italic (Win), Underline and Strikethrough (Mac); Alignment; **Padding (Win)**; Opacity; **Shaded (Win)**, which blends outline and fill; **Line Style** solid, dashed or dotted, plus **No Line (Mac)**;
  - Line Spacing (Mac 2024.2.5).
- Mac-only variants: curved tails, adjustable tail width, rounded corners (shown differently on Win).
- Callouts scale proportionally when the image is scaled down (Mac 2020.1.0).
- Sources: callout; add-comments-labels; version-compatibility; Win 2020/2022/2023 and Mac 2020/2024/2025 histories; features/speech-bubble.

### 3.5 Shape — Both (Control+U on Mac)
- **Rectangle** and **Ellipse** by dragging; Shift gives a square or circle.
- **Polygon**: click to place corners; finish with a double-click (Win) or Enter (Mac).
- Rounded rectangles exist (?). Mac changed its rounded-corner rendering in 2020.0.1 and supports corner radius on multiple objects (2026.1.0).
- Handles: white resize, green rotate.
- **Properties:**
  - Fill (Transparent fill = an empty frame; gradient on Mac);
  - Outline;
  - Shape style;
  - Shadow;
  - **Thickness (Win) / Line Width (Mac)**;
  - Opacity;
  - **Line Style**, including dotted and dashed (Win 2022.1.1; Mac dashed and dotted are lost when opened on Win).
- Sources: shape; Win 2022 history; version-compatibility.

### 3.6 Stamp — Both (Control+I on Mac)
- Pick a stamp from the Quick Styles pane by category dropdown or keyword search, click the canvas to place it, then drag to move or resize with handles.
- **Download More** opens Assets for free and premium stamp sets.
  - Win: a set is sent with "Send to Snagit".
  - Mac: download, then Open.
- **Custom stamps** from JPG, PNG or GIF:
  - Win: Organize Stamps > Add Stamp, into "My Stamps".
  - Mac: Import Stamps, into "Custom".
  - Mac also has **File > Save As Stamp** (Control+Option+Cmd+S, from 2023.2.0).
- Mac: edit a stamp's search keywords (Control-click > Edit Search Terms).
- Keyword search works in several languages.
- Properties: Shadow (off by default since 2024), Opacity.
- Stamps keep their aspect ratio when swapped (Win 2019.1.1) (?).
- **Duplicate:** Ctrl+click with the Stamp tool (Win).
- Emoji: emoji stamps; Mac 2025.3.0 also lets you drag emoji from the macOS Character Viewer onto the canvas.
- Sources: stamp; assets; hotkeys; Win/Mac 2019, 2023 and 2024 histories.

### 3.7 Step — Both (Control+S on Mac)
- Each click places the next number or letter in sequence.
- Resizing or restyling the first step carries over to later steps.
- **Edit Value:** right-click or Control-click a step, or double-click it, to type any alphanumeric label.
- **Restart Sequence** button. A legacy doc says pressing "1" also resets the sequence (?).
- **Properties:** Fill (transparent allowed since 2022.0.0), Text Color, Shape, Shadow, Opacity, **Type** (the sequence style).
- Type values: the current tutorial shows them only as an image. A legacy Snagit 12 doc lists numbers 1–9999, letters A–Z and letter pairs AA–ZZZZZ (?).
- Step text stays upright when the object is rotated.
- **Duplicate:** Ctrl+click (Win).
- Templates renumber steps when sections are added or removed.
- Sources: step; capture-edit-share-snagit-image; Win 2019/2022/2024 histories; Snagit 12.4.1 PDF; 2023 help PDF.

### 3.8 Highlighter — Both (Control+H on Mac)
- Drag to make a translucent highlight.
- **Properties:**
  - Color; Opacity;
  - **Shape (Mac)**: rectangle, rounded rectangle, oval, polygon (Mac 2023.2.0);
  - **Apply only to text (Mac)**: limits the highlight to detected text (shipped Mac 2026.0.0; experimental "Smart Highlight" from 2024.1.0).
- Source: highlighter; Mac 2023/2024/2026 histories.

### 3.9 Pen — Both (Control+P on Mac)
- Freehand drawing; Shift draws a straight horizontal or vertical stroke (Mac 2022.2.0).
- Strokes are **editable only if** "Create as vector" (Win) or "Create editable line" (Mac) was on before drawing; otherwise they become pixels.
- **Properties:**
  - Color; Shadow; Width; Opacity;
  - **Shape** of the pen tip: several on Win; circle only on Mac, where other tips display as circles;
  - **Antialias (Win)**;
  - **Smooth (Win, circle tip only)**;
  - **Smooth Curves (Mac)**.
- Source: pen; version-compatibility.

### 3.10 Magnify — Both (Control+M on Mac)
- Drag a region to create a magnified lens.
- Drag the **anchor** to place the enlarged view elsewhere so the original stays visible ("offset magnification", 2021.0.0).
- **Properties:**
  - Outline color;
  - **Shape**: circle, square, rectangle, rounded rectangle;
  - **Magnify %**, maximum 800% (Win 2021.3.0);
  - Line Width.
- Snaps to other canvas objects.
- A borderless lens shows a temporary border while being moved.
- This tool magnifies image content; it is different from canvas zoom.
- Sources: magnify; features/magnify-image; Win/Mac 2021 histories.

### 3.11 Spotlight — Mac (Windows uses the Spotlight and Magnify effect, §6)
- Draw a region; everything outside it is dimmed and/or blurred.
- **Shapes:** rectangle, rounded rectangle, oval, polygon (polygon added Mac 2023.2.0).
- **Properties:**
  - Color, the surround tint;
  - Opacity of the tint;
  - **Blur** of the surround (Mac 2021.4.1);
  - **Apply only to text** (Mac 2026.0.0).
- Move, resize and rotate with handles.
- Introduced Mac 2021.4.0.
- Sources: spotlight; Mac 2021/2023/2026 histories.

### 3.12 Blur — Both (Control+B on Mac)
- Drag a region to obscure it.
- **Type:**
  - **Blur**, called "Smooth" in older docs;
  - **Pixelate**;
  - **Solid shapes**, opaque redaction and the most secure option.
- **Color** applies to Solid shapes only.
- **Shape** dropdown, including polygon. The 2024 help says Mac only; current docs do not restrict it (?).
- **Intensity** slider; Mac maximum 200 since 2019.
- **Corner radius** (Mac fix 2022.0.1).
- **Apply only to text** (Mac 2026.0.0).
- **Smart Redact** action button (§5.3).
- Security notes from the docs:
  - Blur and pixelation can be reversed by third-party tools, so use Solid shapes for confidential data.
  - Flatten before sharing; Share Link and export to JPG/PNG/GIF/PDF flatten automatically.
- Sources: blur; hide-sensitive-information; 2024 help PDF; Mac 2019/2022 histories.

### 3.13 Simplify — Both (Control+Y on Mac)
- Replaces text and UI detail with plain shapes, for localization, keeping images current when UIs change, and focusing attention.
- **Auto Simplify** toggle overlays simplified objects automatically.
  - The **Detail** slider sets sensitivity.
  - Changing Detail regenerates the objects and **discards manual edits** to them.
- **Color Palette:** edit each palette color; gear > Save Palette as New or Replace. Saved palettes appear in a dropdown.
- **Quick Style objects:** draw extra simplified elements from the Quick Styles pane.
- **Per-object properties:**
  - Fill; Shape;
  - **Background** (text objects); **Text Height**; **Line Spacing**;
  - **Align** left, middle or right (only with several objects selected).
- **Draw/Move mode:** Draw adds objects; Move multi-selects and moves them.
- **Lock Objects (Win)** prevents edits after switching tools.
- **Show Original** shows the source image under the objects, with an **Opacity** slider.
- **Select All (Shape)** for simplify shapes.
- Transparency support (Mac 2020.2.0).
- Introduced 2019.0.0; "Simplify 2.0" (palettes, Draw/Move) in 2021.0.0.
- Sources: simplify; features/image-simplifier; 2019/2021 histories.

---

## 4. Pixel-editing tools (act on the flattened bitmap)

### 4.1 Move — Both (Control+V on Mac)
- Default selection tool: click to select an annotation, drag to move, Delete to remove, double-click to edit its text or properties.
- The **Smart Move** toggle (Win) or button (Mac 2024.4.0) lives here (§5.1).
- **Add Narration** button (Mac 2024.4.0) records microphone audio over the selected tray images to make a video.
- **Duplicate:** Ctrl+click (Win) or Option-drag (Mac).
- Sources: move; Mac 2024 history; hotkeys.

### 4.2 Selection — Both (Control+L on Mac)
- **Shapes:** Rectangle (Shift = square), Ellipse (Shift = circle), Polygon (click corners, double-click to finish), Freehand (lasso).
- **Background Fill** decides what fills the hole when a selection is moved or deleted:
  - **Transparent** (shows the canvas);
  - **Auto-Fill** (content-aware; Win offers **Optimize for Speed** or **Optimize for Photo Quality**).
- **Snap to Object** tightens the selection to the edges of text, icons or shapes inside it; a slider changes which object is chosen.
- **Actions on a selection:**
  - move; Delete; copy and paste (Ctrl/Cmd+C/V); cut (?);
  - right-click **Edit Text**, **Grab Text**;
  - apply **Effects** to the selection only (Win);
  - **Invert Selection** (Ctrl+Shift+I or Shift+Cmd+I);
  - **Print Selection (Mac)**;
  - Save Selection As (Win, mentioned in a fix).
- Cropping to a circle is done with an ellipse selection.
- Edge cases: ellipse, polygon and freehand selections used to copy as rectangles (fixed Win 2026.0.0); the selection is released after a resize (Win 2023.2.1).
- Sources: selection-tools; remove-image-parts; remove-background; Win 2023/2024/2026 histories.

### 4.3 Magic Wand — Both (Control+W on Mac)
- Click to select a region by color similarity.
- **Select** slider (tolerance, a percentage): Less selects fewer pixels, More selects more.
- **Global Color Selection** selects every pixel of that color in the image.
- Uses the same **Background Fill** options as Selection (including the Win Optimize modes).
- Right-click or Control-click > Delete, Cut, Copy; drag to move the selection.
- Best on flat colors; poor on gradients and textures.
- Present in 2022–2026 per the previous-version table.
- Sources: magic-wand; snagit-previous-version-help.

### 4.4 Fill — Both (Control+F on Mac)
- Click to flood-fill the clicked pixel and the connected pixels of similar color.
- **Properties:**
  - Fill color (theme, palette, Hex/RGB, eyedropper, **Transparent**, gradient on Mac);
  - **Tolerance** (1 = exact color only);
  - Opacity;
  - **Global Fill (Mac 2021.4.0)** replaces all matching pixels in the image.
- Works on floating images; clicking an annotation or floating image does not fill the canvas (Win 2026.2.0).
- Primary documented way to make a solid background transparent.
- Sources: fill; remove-background; features/background-remover; Mac 2021 and Win 2026 histories.

### 4.5 Eraser — Both (Control+E on Mac)
- Drag to erase flattened pixels down to the **canvas color** (checkerboard if the canvas is transparent). Annotations are not affected.
- Shift erases a straight line.
- **Properties:** Width; **Opacity (Win)**; **Antialias (Win)**; **Shape circle or square (Mac 2024.4.0)**.
- Mac: the `[` and `]` keys change the eraser size (2019.0.0).
- Sources: eraser; Mac 2019/2024 histories.

### 4.6 Crop — Both (Control+C on Mac)
- Drag the blue handles to define the area to keep; drag the box to reposition; press the **Crop** button to apply.
- `.snagx` keeps cropped-away pixels by default, so crops can be reversed with **Revert to Original** (Edit menu).
- Win has a preference **Delete cropped content** to discard them.
- Related: **Crop to Canvas** (Mac Cmd+K), **Trim** (Mac Shift+Cmd+X), **Auto Trim** (Win; removes uniform-color or transparent borders).
- Aspect-ratio presets are not documented (?).
- Sources: crop; features/crop-image; preferences; hotkeys; Snagit 13 PDF; Win 2026 history.

### 4.7 Cut Out — Both (Control+X on Mac)
- Removes a band spanning the **full width (horizontal) or full height (vertical)** and closes the gap.
- The direction and edge style come from the chosen Quick Style:
  - **Straight** edges stitch the halves together seamlessly.
  - Other edge styles (torn or wave-like (?)) leave a gap whose size cannot be edited.
- Using a Horizontal or Vertical style inside an existing gap reduces it.
- **Add Horizontal Space / Add Vertical Space (Mac 2023.1.0)** inserts a band of a dragged pixel size.
- Mac shows the pixel size of the band while dragging (2019.1.3).
- **Requires flattening existing edits first**, with a prompt.
- Effects are disabled while Cut Out is active (Win 2024.0.0).
- Sources: cut-out; remove-image-parts; features page; Mac 2019/2023 and Win 2024 histories.

---

## 5. Smart, OCR and ML features (all processed on the device)

### 5.1 Smart Move — Both
- Turn it on with the Move tool (Win toggle; Mac button since 2024.4.0).
- Detects UI elements and text in the flattened image and turns them into movable objects; handles appear on hover.
- **Detail** slider controls how finely elements are split. Win 2019.1.0 also had a reset button.
- Selected objects can be moved, resized or deleted; double-click a text object to retype it.
- Mac: **Done** flattens and exits.
- Works on floating images too (2019.0.0).
- Uses OpenCV locally; nothing is uploaded.
- Introduced in Snagit 2018 (press coverage).
- Sources: move; support "What is Smart Move" (Win and Mac articles); TechSmith and AI; thejournal; Win 2019 history.

### 5.2 Grab Text and Edit Text (successor of "Text Replace", Snagit 2018) — Both
- **Grab Text** (OCR):
  - Right-click the whole image, or a selection, > Grab Text (Mac shortcut Shift+Cmd+O).
  - A **Grab Text Results** dialog lets you edit the text, choose plain text (Win "Plain text", Mac "Make Plain Text", Win 2024.3.5), and **Copy All**.
  - Mac: grab text from a **paused video frame**.
  - Can also be chosen as a capture-time selection mode (capture team).
  - Results include paragraph spacing (Win 2026.4.0); tall captures and colors are handled better (Mac 2026.1.0).
  - Spell check covers Grab Text output (Win).
- **Edit Text:**
  - Select text with the Selection tool, right-click > Edit Text, then double-click to place a cursor.
  - You can then change color, font and size in Tool Properties, add, replace or delete text.
  - The original pixels are replaced by an auto-filled background (Mac 2019 fix) (?), and the original color is kept (Win fix 2024.1.4).
  - Include the whole text in the selection and keep background objects out of it, to avoid blurring.
- **OCR engines:** ABBYY and Windows Media OCR on Win; Apple OCR on Mac.
- **Languages:** Spanish and Portuguese were added in 2021.0.1. The full language list is not documented (?).
- Experimental improved OCR on Win (2026.0.0; "Optimized OCR" 2026.3.0).
- Grab Text needs macOS 10.15 or later.
- Sources: edit-image-text; TechSmith and AI; Win 2021/2024/2026 and Mac 2022/2026 histories; support 4403309311757; thejournal.

### 5.3 Smart Redact — Both
- Opened from Blur tool > **Smart Redact** in the Actions panel. Introduced 2025.0.0 (Mac experiment 2024.2.3).
- **Detects 9 categories:** addresses, credit-card numbers, dates, email addresses, faces, IP addresses, phone numbers, SSNs, URLs.
- Turn categories on or off; turning one off removes its redactions.
- **Type:** Solid shapes (black by default), Blur, or Pixelate.
- Solid **Shape Color** is adjustable (2025.1.0 Win; 2025.2.0 Mac). Redactions have rounded corners.
- **Done** closes the panel. Each redaction remains a normal Blur object you can delete or restyle (type, color, shape, intensity).
- Mac 2026.0.0 additions:
  - click text to add or remove a redaction;
  - right-click > **Redact All Matches**, Remove, Edit Text;
  - find similar sensitive content across multiple screenshots.
- Improvements: side-profile faces (2025.1.0), addresses (Mac 2025.3.1), performance with many redactions (Mac 2026.3.0).
- **Engines:** Win ABBYY plus an OpenCV face classifier; Mac Apple OCR plus Apple person and face detection. Local only; nothing is uploaded.
- Sources: blur; hide-sensitive-information; features/smart-redact; Win/Mac 2025–2026 histories; whats-new changelog.

### 5.4 Remove Background (automatic) — Mac
- Right-click or Control-click an image > **Remove Background**. Works on photos and non-solid backgrounds.
- Weak on fine detail such as hair; clean up with the Eraser.
- Uses Apple Subject Detection / Person Segmentation.
- Mac 2025.0.0 ("Remove Backgrounds & Create Perfect Cutouts").
- Windows relies on the manual methods: Fill Transparent, Selection + Invert + Delete, Magic Wand + Global + Delete.
- Sources: remove-background; TechSmith and AI; whats-new changelog page 2.

### 5.5 Auto Simplify — Both
See §3.13.

### 5.6 Smart text targeting ("Apply only to text") — Mac
- Available on Highlight, Blur and Spotlight. Shipped 2026.0.0 (experiment 2024.1.0).
- Source: Mac 2024 and 2026 histories.

### 5.7 Translate Text — Mac, REMOVED
- Extracted text from Text and Callout objects into an **XLIFF** file and re-imported translations to generate localized images.
- Removed in Mac 2024.2.5. Listed for historical parity only.
- Source: blog translate-text.

---

## 6. Image-level effects (Effects panel)

**General behavior**
- **Applying:**
  - Win: pick an effect, adjust it in its dropdown, then click Apply.
  - Mac: click to toggle on, and use the gear for settings.
- **Quick Effects** are preset combinations; you can save your own (Win "Add to Quick Effects", Mac +).
- **Removing:**
  - Win: Undo.
  - Mac: click the active (blue) effect to toggle it off, or choose "No Effects".
- Win can apply an effect **to a selection only**.
- Effects apply to floating images too (Win 2021.1.0).
- Effects can also be applied at capture time and in batch export.
- **Mac effects are not kept when a `.snagx` is re-saved on Windows** (§10).
- Sources: effects; blog effects; 2021 help PDF.

| Effect | Options | Platform | Source |
|---|---|---|---|
| **Border** | Color, width | Both | effects |
| **Edges** | Style dropdown (torn edge confirmed; others such as wave or bevel (?)), Size, which sides; Win adds shadow color and location. One edge effect at a time (legacy). | Both | effects; Snagit 13 PDF; how-to-edit-a-screenshot |
| **Shadow** | Direction grid (one side or all around; can be centered), color, size. Mac adds opacity, offset and blur. Enlarges the canvas. | Both | effects; Win 2019 and Mac 2024 histories |
| **Perspective** | Style dropdown (legacy Win: Perspective or Shear); Horizontal/Rotation and Vertical/Tilt sliders. Legacy Mac also had border and shadow. | Both | effects; Snagit 13 PDF; Mac 2018 PDF |
| **Page Curl** | Corner and size (legacy) | Win | effects; Snagit 13 PDF |
| **Reflection** | Size (% reflected), opacity | Mac | effects |
| **Fade** | Gradually dims the edges; size and intensity (legacy) | Mac | effects; Mac 2018 PDF |
| **Filters** | Soften, Sharpen, Opacity, Monochrome, Grayscale, Invert Colors | Win | effects |
| **Color Filters** | Grayscale, Sepia Tone, Invert, Sharpen, Opacity (Opacity added Mac 2020.1.0) | Mac | effects; Mac 2020 history |
| **Color Adjustment** | Brightness, Contrast, Hue, Saturation, Gamma, plus reset to defaults | Both (Mac 2022.1.0) | effects; Mac 2022 and Win 2023 histories |
| **Color Replacement** | Swap two colors, or replace one with another; transparent swatch (2020.1.3) | Win | effects; Win 2020 history |
| **Spotlight and Magnify** | Needs a selection; Image > Effects > Spotlight and Magnify, adjust, Apply. Options not documented (?). | Win | effects |
| **Watermark** | Import an image; position grid; Scale; **Underlay** (embossed into the background) or **Overlay** (on top) | Win | effects; Snagit 13 PDF |
| **Capture Info** | Fields: application name and version (on Win the URL replaces the browser name), date and time, OS, build number, **system username (Mac 2024.0.0)**, **custom text** (Mac can prompt for it). Background color; font, color and size. Only for images captured in Snagit. | Both | effects; add-capture-information; blog effects; Mac 2019/2023/2024 histories |
| **Trim** (as an effect) | Removes uniform borders | Mac 2022.1.0 | Mac 2022 history |
| **Corner rounding** (as a capture effect) | Preset corner radius applied at capture | Mac 2025.2.1 | Mac 2025 history |

---

## 7. Image and canvas operations

| Name | Behavior | Platform | Version | Source |
|---|---|---|---|---|
| **Resize Image** | Image menu, or click the dimensions (Mac Shift+Cmd+R). Units: Percent, Pixels, Inches, Centimeters. Aspect lock. Win Advanced: **Resolution (DPI)** and **Use Resampling** (warns about quality loss). Mac: **Pixel Density** (PPI; 1 shows all pixels). Flattened objects can no longer be edited afterwards. | Both | Mac inches/cm 2019.1.3; Mac pixel resolution 2022.1.0 | resize-rotate-flip-image; Mac histories |
| **Resize Canvas / extend canvas** | Drag a canvas edge handle to add space. Mac: resize to an exact size (2023.2.0). Cmd-drag resizes from the center. Preference **Auto-expand canvas to fit objects** that are pasted or dragged past the edge. | Both | — | how-to-edit-a-screenshot; Mac 2023/2025 histories; preferences |
| **Rotate** | Win: 90° clockwise, 90° counter-clockwise, 180°, Custom Angle (positive integers). Mac: dialog with angle field, dial and a 90° button (Cmd+R). | Both | — | resize-rotate-flip-image |
| **Flip** | Image > Arrange > Flip Vertical or Horizontal; also flips annotations and text. | Both | Mac 2021.0.1 | resize-rotate-flip-image; Mac 2021 history |
| **Canvas color** | Image > Canvas Color (Win) or Change Canvas Color (Mac). Any color, Transparent (checkerboard), or gradient on Mac. Erased and transparent areas show it. | Both | gradient Mac 2025.0.0 | remove-background; eraser |
| **Rounded image corners** | Hover over the canvas and drag the **yellow diamond** at the top-right. Clicking it applies a 20 px radius; dragging adjusts it. Dragging to the middle of the top edge turns a square into a circle; save as PNG or GIF to keep transparency. Works with Border, Capture Info, Perspective and Shadow effects (Mac experiments). Undoable (Mac 2025.3.1). Also on floating images (Mac experiment 2024.1.0), and on multiple objects at once (Mac 2026.1.0). | Both | Win 2024.3.0; Mac 2024.4.0 | how-to-edit-a-screenshot; remove-image-parts; histories |
| **Trim / Auto Trim / Crop to Canvas** | See §4.6. | Win (Auto Trim) / Mac (Trim, Crop to Canvas) | — | hotkeys; Snagit 13 PDF |
| **Revert to Original** | Discards all edits and goes back to the capture as taken; also undoes Cut Out. | Both (?); Mac toolbar button 2020.1.3 | — | features/crop-image; Mac histories |
| **Image DPI and pixel density** | Win Resolution field in Resize Advanced; Mac Pixel Density and Retina scaling options. A legacy doc says the default is 96 DPI. | Both | — | resize-rotate-flip-image; preferences-mac |

---

## 8. Canvas composition and object management

| Name | Behavior | Platform | Source |
|---|---|---|---|
| **Floating images (combine images)** | Drag from the tray, paste, or drag from a browser or Finder onto the canvas to create a movable image object. Pasted transparent clipboard images keep transparency. Preference **Automatically flatten pasted images** (Win). | Both | how-to-edit-a-screenshot; preferences; Win 2021 history |
| **Multi-select** | Select All (Ctrl/Cmd+A); Mac also "Select All Objects" (Option+Cmd+A) and Edit > Select All (2023.2.2). Deselect All (Ctrl/Cmd+D). Tool-scoped Select All (§2.7). Marquee and Shift-click behavior not documented (?). | Both | hotkeys; Mac 2023 history |
| **Multi-object transform** | Resize and rotate several selected objects together, including floating images; corner radius for several at once. | Mac 2026.1.0 | Mac 2026 history |
| **Z-order** | Bring Forward (Ctrl+Shift+F / Cmd+↑), Send Backward (Ctrl+Shift+B / Cmd+↓). Bring to Front and Send to Back with Option+Cmd+↑/↓ (Mac). | Both | hotkeys |
| **Group / Ungroup** | Ctrl+Shift+G / U (Win); Shift+Cmd+G / U (Mac; grouping arrived on Mac in 2022.0.0). Grouped annotations cannot be saved to legacy `.snag` (warning). | Both | hotkeys; Mac 2022 and Win 2023 histories |
| **Align / distribute** | Mac has Align options including distribute "Evenly" within the selection (2024.2.5 fix; renamed 2025.4.0). Simplify objects align left, middle or right on both platforms. General alignment on Win is not documented (?). | Mac (general); Both (Simplify) | Mac 2024/2025 histories; simplify |
| **Canvas Snapping** | Image menu toggle (Ctrl+; / Cmd+;). Objects snap to edges and centers of the canvas and other objects (?). Snap lines respect Increase Contrast on Mac. | Both | support 360029204112; hotkeys; Mac 2020 history |
| **Lock objects** | Mac objects can be locked; the lock is ignored on Win. Template image areas use a lock icon on Mac. | Mac | version-compatibility; modify-template |
| **Exact position and size** | **Not supported.** No numeric X/Y or W/H fields for objects or selections; everything is done by dragging. | Win (article) | support 45792605008525 |
| **Nudging** | Arrow-key movement of objects is implied by a fix for objects moved off-canvas with the keyboard; step size undocumented (?). | Win (?) | Win 2019 history |
| **Duplicate object** | Ctrl+click with Move, Stamp or Step (Win); Option-drag or Control-click > Duplicate (Mac 2023.2.3). | Both | hotkeys; Mac 2023 history |
| **Cut / Copy / Paste** | Standard shortcuts. **Paste in Place** (Mac Shift+Cmd+V, 2022.2.0); **Paste and Match Style** for text (Mac). **Copy All** copies the whole flattened image (Ctrl+Shift+C / Option+Cmd+C). | Both | hotkeys; Mac 2022 history |
| **Flatten / Flatten All** | Merges one object (Ctrl+T / Shift+Cmd+T) or all objects (Ctrl+Shift+T / Control+Shift+T) into pixels; they are no longer editable. Also on the right-click menu. **Unflatten** an object or all objects (Mac 2024.3.2). | Both (Unflatten Mac) | hotkeys; blur; Mac 2024 history |
| **Undo / Redo** | Ctrl+Z, Ctrl+Y / Cmd+Z, Shift+Cmd+Z. Multi-level; undo covers corner rounding (Mac 2025.3.1); faster with many objects (Mac 2026.0.0). No history panel documented (?). | Both | hotkeys; histories |
| **Cursor object** | A captured cursor is stored as a separate vector object that can be moved, resized or deleted in the editor. | Both | image-capture; features page ("Edit cursor") |

---

## 9. Create Image from Template, and combining images — Both

- **Entry points:**
  - Create > Image from Template (Ctrl+Alt+T / Option+Shift+T).
  - With several tray images selected: **Combine in Template** (Ctrl+Alt+C / Option+Shift+C).
  - **Apply Template** to a single image.
- Template picker can filter templates by the number of selected images. A maximum of 10 images was enforced in Win 2020.1.5.
- **Template families:**
  - Numbered steps or how-to (including "Custom Steps", the successor of the 2019 Combine Images feature);
  - Timeline;
  - Comparison (side by side, before and after).
  - Free Starter templates; premium ones via an Assets subscription. Individual template names are not listed in public text (?).
- **Template Settings:**
  - **Image Scale**: Fill entire drop zone (may crop) or Fit entire image (may leave space);
  - **Footer**: "Footer Author Name" (Win) or "Created By" (Mac), set to System (computer display name) or Custom;
  - title; per-image captions; theme; font and color; canvas background color;
  - **Number Images** (adds Step objects, with a color option);
  - reverse the order of selected images (Mac 2023.2.0).
- **Drop zones:**
  - drag or paste images into them; drag a floating image in;
  - double-click to reposition the image inside;
  - **zoom in/out buttons** in 10% steps (2025.0.0);
  - images snap to the sides and center of the zone;
  - **Drag to Swap** moves an image and its caption to another zone;
  - right-click to delete an image or caption;
  - **Capture** button inside an empty placeholder (Mac 2025.1.0);
  - empty placeholders are omitted when saving, printing or copying.
- **Sections:** add (choose a content type), resize with the blue divider, delete with an icon. Related objects reflow when a section is resized. Mac needs the lock icon to edit image areas and text boxes.
- **Apply theme colors to template objects:** click an object, optionally use Select All, then choose a theme in Quick Styles and a style or palette color.
- **Output:** a normal editable image. Save it as `.snagx` to reuse it as your own template. Print or PDF output gets page breaks or multi-page splitting.
- Shape objects can be drawn over drop zones (Win 2025.0.0).
- Introduced 2020.0.0; "Templates 2.0" in 2021.0.0.
- Sources: image-from-template; modify-template; template-step-by-step-instructions; Win 2020/2021/2024/2025 and Mac 2020/2023/2025 histories.

---

## 10. Files: new, open, save, export, compatibility

### 10.1 New and open
- **New Image** (Ctrl/Cmd+N): a blank canvas. Mac lets you set default dimensions and canvas color in Advanced settings.
- **New from Clipboard** (Ctrl+Shift+N / Shift+Cmd+N).
- **Open** (Ctrl/Cmd+O); drag and drop from the file manager or a browser.
- **Other import routes:**
  - Mac: macOS screenshots and AirDrop files open in Snagit (settings); iPhone/iPad images via **Continuity Camera** (2025.2.0).
  - Win: mobile import ("Mobile Sharing"), Windows Snipping Tool integration, Printer Capture to the editor (multi-page documents).
  - Both: Camtasia "Edit in Snagit" round trip (Win 2025.2.0+; Camtasia 2025.2.0+).
- **SVG cannot be opened**; convert it to PNG or PDF first.
- Mac opens MOV directly (2025.3.0).
- Sources: hotkeys; preferences-mac; support SVG article; histories; features page.

### 10.2 `.snagx` native editable format — Both, from 2022.0.0
- Cross-platform. It replaced `.snag` (Win v8–2021) and `.snagproj` (Mac v1–2021).
- Keeps vector objects (text, callouts, stamps, shapes, pasted images, cursor), effects, crop history and capture metadata.
- Every image capture **auto-saves to the Library** as `.snagx`. Win can turn this off; unsaved captures are then deleted when Snagit closes.
- Converting old `.snag` / `.snagproj` files: required on Win, optional on Mac; makes backups.
- **Save As `.snag`** drops: T arrow ends, multiple and cloud tails, dashed or dotted callout lines, reduced padding, transparent step fills.
- **Windows to Mac:** some properties render differently but are preserved — some end styles, curve handles, rounded callouts, curved tails and tail widths, non-circle pen tips, text padding.
- **Mac to Windows:** some properties may be lost — effects (border, shadow, edges), object locks, curve handles, Global Fill, explosion-callout tails, dashed or dotted shape lines. A warning dialog offers Open File or Cancel.
- Newer Step Capture `.snagx` files need current versions.
- Sources: file-save-tips; version-compatibility; convert-previous-captures; Win 2022 history.

### 10.3 Save As and export formats
- **Win:** SNAGX, SNAG, BMP, CUR, EMF, EPS (grayscale; export only), GIF, ICO, JPG, MHT, PDF (export), PNG, PSD, RAS, TGA, TIF, WFX, WMF, WPG, **WebP** (2022.1.1).
- **Mac:** SNAGX, SNAGPROJ, BMP, GIF, **HEIF/HEIC** (2022.1.1 edit; Save As 2023.2.2), JPG (`.jpg` extension since 2024.3.0), PDF (export), PNG, TIFF, **WebP** (2022.2.0).
- **Transparency is kept in:** CUR, GIF, ICO, PNG, TIF, SNAGX.
- **Per-format options:** Win "Save As options match each format" (2023.1.0); TIFF compression (Mac fix 2026.0.0); multi-page TIFF (Win, page delete fix 2023.2.1); multi-page PDF, including scrolling captures paginated to PDF (Mac 2026.4.0) and templates split across pages. JPG quality and similar settings are not documented (?).
- **Default format:**
  - Win: Save As Preferences — remember the last format, or always use a chosen one.
  - Mac: "Default sharing format" for Export, Copy All, Share Link and drag-out.
- **Drag-out file type:**
  - Win: "Drag and drop file type" preference (2024.1.2).
  - Mac: Retina downscaling when sharing; holding Shift skips it.
- **After saving to another format:**
  - Win: the exported file stays editable while in the tray.
  - Mac: the non-SNAGX copy is flattened on quit. A setting keeps a duplicate editable `.snagx`.
- **Other file commands:**
  - Save (Ctrl/Cmd+S), Save As (Ctrl+Shift+S / Shift+Cmd+S), Save All (Mac);
  - Close (Ctrl+W / Shift+Cmd+W), Close All (Mac);
  - Rename (F2), Duplicate, Delete or Move to Trash;
  - Reveal in Library (Mac);
  - **File > Export** (Mac; exports unsaved edits);
  - **Batch Export / Batch Conversion Wizard** (§10.4).
- Sources: file-save-tips; support 41688493993613; support 44545405511821; features page; preferences; preferences-mac; histories.

### 10.4 Batch convert / export — Both
- **Win wizard** (File > Export, or Library right-click > Batch Conversion Wizard), in steps: files > effects > location > format > naming (automatic filename with prefix and numbering options).
- **Mac dialog:** effects with a gear for settings, **rotation** (Mac only), format, a base filename with `_1`, `_2` suffixes, and Reveal in Finder.
- Mac can also batch-export Step Capture images (2026.1.0).
- Source: batch-convert-images; Mac 2026 history.

### 10.5 Print — Both
- Print (Ctrl/Cmd+P); Page Setup (Mac Shift+Cmd+P); Print Selection (Mac).
- Mac remembers the paper size (2023.2.3).
- Templates paginate when printed.
- Sources: hotkeys; selection-tools; Mac 2023 and Win 2024 histories.

---

## 11. Editor preferences that change editor behavior

- **Win, Editor tab:**
  - workspace color;
  - tray thumbnail size;
  - auto-expand canvas;
  - Escape closes Editor;
  - cursor Quick Style popup;
  - toolbar labels;
  - spell check and its language (EN-UK/US, FR, DE, PT, ES).
- **Win, Advanced tab:**
  - keep Editor running in the background;
  - **Delete cropped content**;
  - **Auto-flatten pasted images**;
  - drag-and-drop file type;
  - reset to defaults.
- **Win, Experiments tab** (examples):
  - HDR Capture Fidelity (2025.4.0);
  - Quick Image Annotation — annotate and copy to the clipboard in one step (2026.1.0);
  - improved / Optimized OCR (2026.0.0 / 2026.3.0).
- **Win, Library tab:**
  - tray count 1–200;
  - auto-save images and videos;
  - prompt before deleting;
  - remove unpinned items on close.
- **Mac, Advanced:**
  - scale down Retina images when sharing;
  - keep a duplicate editable capture when saving;
  - auto-expand canvas;
  - **auto-select annotations after drawing**;
  - show quick edit toolbar;
  - default new-image size and color;
  - default sharing format.
- **Mac, Experiments** (examples):
  - Live Measure — pixel-distance measurement (2024.1.0);
  - Smart Highlight;
  - canvas corner radius;
  - **Screen Draw tools available anytime** (2026.3.0);
  - QR codes (2025.4.0).
- Sources: preferences; preferences-mac; Win 2025/2026 and Mac 2024–2026 histories.

---

## 12. Keyboard shortcuts (editor)

**Mac tool shortcuts:**

| Tool | Shortcut | Tool | Shortcut |
|---|---|---|---|
| Favorites | Control+R | Blur | Control+B |
| Arrow and Line | Control+A | Simplify | Control+Y |
| Text | Control+T | Magic Wand | Control+W |
| Callout | Control+D | Crop | Control+C |
| Shape | Control+U | Cut Out | Control+X |
| Stamp | Control+I | Pen | Control+P |
| Fill | Control+F | Highlighter | Control+H |
| Move | Control+V | Step | Control+S |
| Selection | Control+L | Eraser | Control+E |
| Magnify | Control+M | | |

**Windows tool shortcuts:** Ctrl+1 to Ctrl+9 select tools by toolbar position.

**Editing and view shortcuts** (Win / Mac):

| Action | Windows | Mac |
|---|---|---|
| Undo | Ctrl+Z | Cmd+Z |
| Redo | Ctrl+Y | Shift+Cmd+Z |
| Cut / Copy / Paste | Ctrl+X / C / V | Cmd+X / C / V |
| Copy All | Ctrl+Shift+C | Option+Cmd+C |
| Paste in Place | — | Shift+Cmd+V |
| Paste and Match Style | — | Option+Shift+Cmd+V |
| Select All | Ctrl+A | Cmd+A |
| Deselect | Ctrl+D | Cmd+D |
| Invert Selection | Ctrl+Shift+I | Shift+Cmd+I |
| Group | Ctrl+Shift+G | Shift+Cmd+G |
| Ungroup | Ctrl+Shift+U | Shift+Cmd+U |
| Bring Forward | Ctrl+Shift+F | Cmd+↑ |
| Send Backward | Ctrl+Shift+B | Cmd+↓ |
| Bring to Front / Send to Back | — | Option+Cmd+↑ / ↓ |
| Flatten | Ctrl+T | Shift+Cmd+T |
| Flatten All | Ctrl+Shift+T | Control+Shift+T |
| Canvas Snapping | Ctrl+; | Cmd+; |
| Actual Size | Ctrl+0 | Cmd+0 |
| Zoom to Fit | — | Shift+Cmd+0 |
| Zoom In / Out | Ctrl +/− | Cmd +/− |
| Resize Image | — | Shift+Cmd+R |
| Rotate | — | Cmd+R |
| Crop to Canvas | — | Cmd+K |
| Trim | — | Shift+Cmd+X |
| Grab Text | — | Shift+Cmd+O |
| Save As Stamp | — | Control+Option+Cmd+S |
| Image from Template | Ctrl+Alt+T | Option+Shift+T |
| Combine in Template | Ctrl+Alt+C | Option+Shift+C |
| Video from Images | Ctrl+Alt+V | Option+V |
| Share Link | Ctrl+Shift+L | Option+Cmd+L |

**Modifier keys while drawing:**
- **Shift** constrains (square, circle, straight line).
- **Option-drag** (Mac) draws over objects or duplicates the selection.
- **Ctrl** (Win) switches between the tool and the color picker.

Source: snagit-hotkeys.

---

## 13. Newly added editor items, 2024–2026 (parity checklist)

**2024**
- Starter theme and refreshed stamps, with stamp shadow off by default (2024.0.0).
- Tray pinning (Win 2024.0.0; Mac 2026.1.0).
- Rounded image corners (Win 2024.3.0; Mac 2024.4.0).
- Assets button (Win 2024.3.0; Mac 2024.3.2).
- Unflatten (Mac 2024.3.2).
- Square eraser (Mac 2024.4.0).
- Add Narration (Mac 2024.4.0).
- Line Spacing for Text and Callout (Mac 2024.2.5).
- Translate Text removed (Mac 2024.2.5).

**2025**
- Smart Redact (2025.0.0, both), with color, rounded corners and better detection in 2025.1.x.
- Remove Background (Mac 2025.0.0).
- Gradients (Mac 2025.0.0).
- Template drop-zone zoom buttons and shapes over drop zones.
- Cmd-drag resize from center (Mac).
- Select All per tool (Mac 2025.3.0).
- Emoji drag-in (Mac 2025.3.0).
- MOV import (Mac 2025.3.0).
- Continuity Camera import (Mac 2025.2.0).
- Corner rounding at capture (Mac 2025.2.1).
- Edit in Snagit from Camtasia (Win 2025.2.0).
- In-app Assets (Mac 2025.4.0).
- Help menu in the editor.

**2026**
- Apply-only-to-text for Highlight, Blur and Spotlight (Mac 2026.0.0).
- Redesigned color picker (Mac 2026.0.0).
- Smart Redact click-to-redact and Redact All Matches (Mac 2026.0.0).
- Select All per tool (Win 2026.1.0).
- Multi-object resize, rotate and corner radius (Mac 2026.1.0).
- Auto-select after drawing (Mac 2026.1.0).
- Fill on floating images (Win 2026.2.0).
- System theme option (Win 2026.2.0).
- UI refresh (Win 2026.3.0; Mac 2026.0.0).
- Arrow keys adjust sliders (Win 2026.0.0).
- Grab Text paragraph spacing (Win 2026.4.0).
- PDF pagination of scrolling captures (Mac 2026.4.0).
- Experiments: improved OCR, Quick Image Annotation, Screen Draw tools anytime.

**AI / ML (all local)**
- Smart Redact, Smart Move, OCR (Grab Text, Edit Text), face detection, background removal (Mac), Auto Simplify.
- The TechSmith and AI page marks none of them as generative.
- The only generative feature is AI image generation on the Assets website (premium); it is not part of the editor.

---

## 14. Adjacent items owned by other teams (listed only for boundary clarity)
- **Step Capture guide editing:**
  - steps anchor callouts, stamps and arrows;
  - the cursor can be moved or deleted;
  - export to Word, PowerPoint, HTML or Markdown.
- **Video trim and combine; Video from Images; Screen Draw.**
- **Library, tags, Share destinations, Share Link.**

---

## 15. Open questions (not confirmed in public text)
1. Names of the Edges effect styles and the Cut Out edge styles (torn and wave are suggested by marketing; sawtooth and bevel are unverified).
2. Full lists of Callout shapes, Shape-tool shapes, Step Types, and Arrow end and line styles. Current docs show these only as images.
3. Whether Windows has general object Align/Distribute and Lock, a marquee multi-select, and arrow-key nudge step sizes.
4. JPG quality, PNG and GIF options, and color depth in Save As.
5. The full OCR language list.
6. Options inside the Windows Spotlight and Magnify effect.
7. Crop aspect-ratio presets, if any.
8. Whether Mac has a separate Line tool or only Arrow with line end styles.

---

## Sources consulted (deduplicated)

**TechSmith Learn tutorials (www.techsmith.com/learn/tutorials/...)**
- https://www.techsmith.com/learn/tutorials/snagit/
- https://www.techsmith.com/learn/tutorials/section/snagit/snagit-tools/
- https://www.techsmith.com/learn/tutorials/section/snagit/edit-captures/
- https://www.techsmith.com/learn/tutorials/snagit/how-to-edit-a-screenshot/
- https://www.techsmith.com/learn/tutorials/snagit/whats-new/
- https://www.techsmith.com/learn/tutorials/snagit/get-to-know-snagit/
- https://www.techsmith.com/learn/tutorials/snagit/arrow/
- https://www.techsmith.com/learn/tutorials/snagit/blur/
- https://www.techsmith.com/learn/tutorials/snagit/callout/
- https://www.techsmith.com/learn/tutorials/snagit/text/
- https://www.techsmith.com/learn/tutorials/snagit/shape/
- https://www.techsmith.com/learn/tutorials/snagit/stamp/
- https://www.techsmith.com/learn/tutorials/snagit/fill/
- https://www.techsmith.com/learn/tutorials/snagit/move/
- https://www.techsmith.com/learn/tutorials/snagit/selection-tools/
- https://www.techsmith.com/learn/tutorials/snagit/edit-image-text/
- https://www.techsmith.com/learn/tutorials/snagit/simplify/
- https://www.techsmith.com/learn/tutorials/snagit/magic-wand/
- https://www.techsmith.com/learn/tutorials/snagit/crop/
- https://www.techsmith.com/learn/tutorials/snagit/cut-out/
- https://www.techsmith.com/learn/tutorials/snagit/pen/
- https://www.techsmith.com/learn/tutorials/snagit/line/
- https://www.techsmith.com/learn/tutorials/snagit/highlighter/
- https://www.techsmith.com/learn/tutorials/snagit/step/
- https://www.techsmith.com/learn/tutorials/snagit/eraser/
- https://www.techsmith.com/learn/tutorials/snagit/magnify/
- https://www.techsmith.com/learn/tutorials/snagit/spotlight/
- https://www.techsmith.com/learn/tutorials/snagit/effects/
- https://www.techsmith.com/learn/tutorials/snagit/quick-styles-and-favorites/
- https://www.techsmith.com/learn/tutorials/snagit/snagit-themes/
- https://www.techsmith.com/learn/tutorials/snagit/customize-toolbar/
- https://www.techsmith.com/learn/tutorials/snagit/add-capture-information/
- https://www.techsmith.com/learn/tutorials/snagit/image-from-template/
- https://www.techsmith.com/learn/tutorials/snagit/modify-template/
- https://www.techsmith.com/learn/tutorials/snagit/snagit-hotkeys/
- https://www.techsmith.com/learn/tutorials/snagit/preferences/
- https://www.techsmith.com/learn/tutorials/snagit/preferences-mac/
- https://www.techsmith.com/learn/tutorials/snagit/recent-captures-tray/
- https://www.techsmith.com/learn/tutorials/snagit/gradients/
- https://www.techsmith.com/learn/tutorials/snagit/version-compatibility/
- https://www.techsmith.com/learn/tutorials/snagit/remove-image-parts/
- https://www.techsmith.com/learn/tutorials/snagit/remove-background/
- https://www.techsmith.com/learn/tutorials/snagit/hide-sensitive-information/
- https://www.techsmith.com/learn/tutorials/snagit/add-comments-labels/
- https://www.techsmith.com/learn/tutorials/snagit/speed-up-snagit-workflow/
- https://www.techsmith.com/learn/tutorials/snagit/resize-rotate-flip-image/
- https://www.techsmith.com/learn/tutorials/snagit/file-save-tips/
- https://www.techsmith.com/learn/tutorials/snagit/batch-convert-images/
- https://www.techsmith.com/learn/tutorials/snagit/assets/
- https://www.techsmith.com/learn/tutorials/snagit/convert-previous-captures/
- https://www.techsmith.com/learn/tutorials/snagit/image-capture/
- https://www.techsmith.com/learn/tutorials/snagit/capture-edit-share-snagit-image/
- https://www.techsmith.com/learn/tutorials/snagit/video-from-images/
- https://www.techsmith.com/learn/tutorials/snagit/snagit-previous-version-help/
- https://www.techsmith.com/learn/tutorials/academy/guide-for-using-snagit-accessibly/
- https://www.techsmith.com/learn/wp-json/wp/v2/tutorial/315916
- https://www.techsmith.com/learn/wp-json/wp/v2/tutorial/145453
- https://www.techsmith.com/learn/webinars/learn-one-new-thing-snagit-effects/

**TechSmith blog tutorials**
- https://www.techsmith.com/blog/tutorials/snagit/effects/
- https://www.techsmith.com/blog/tutorials/snagit/translate-text/

**Official help PDFs**
- https://www.techsmith.com/learn/wp-content/uploads/2025/01/Snagit-Help-2024-EN.pdf
- https://www.techsmith.com/learn/wp-content/uploads/2023/09/Snagit-Help-2023-EN.pdf
- https://www.techsmith.com/learn/wp-content/uploads/2022/10/Snagit-Help-2022.pdf
- https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-Help-2021.4.pdf
- https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-Windows-Help-2018.1.pdf
- https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-Mac-Help-2018.1.pdf
- https://www.techsmith.com/learn/wp-content/uploads/2021/07/Snagit-13.pdf
- https://assets.techsmith.com/Docs/pdf-snagit/Snagit_12.4.1_Help.pdf
- https://assets.techsmith.com/Docs/pdf-snagit/Creating_Process_Documentation_with_Snagit_Mac.pdf

**Version histories (support.techsmith.com)**
- https://support.techsmith.com/hc/en-us/sections/39094961933453 (the redirect target of techsmith.com/snagit-version-history.html)
- https://support.techsmith.com/hc/en-us/articles/115006443147
- Windows:
  - https://support.techsmith.com/hc/en-us/articles/42674936732685-Snagit-Windows-2026-Version-History
  - https://support.techsmith.com/hc/en-us/articles/39102293598349-Snagit-Windows-2025-Version-History
  - https://support.techsmith.com/hc/en-us/articles/39102250364429-Snagit-Windows-2024-Version-History
  - https://support.techsmith.com/hc/en-us/articles/39102261751949-Snagit-Windows-2023-Version-History
  - https://support.techsmith.com/hc/en-us/articles/43410048034061-Snagit-Windows-2022-Version-History
  - https://support.techsmith.com/hc/en-us/articles/43410747342605-Snagit-Windows-2021-Version-History
  - https://support.techsmith.com/hc/en-us/articles/43410801744525-Snagit-Windows-2020-Version-History
  - https://support.techsmith.com/hc/en-us/articles/43414049971213-Snagit-Windows-2019-Version-History
- Mac:
  - https://support.techsmith.com/hc/en-us/articles/41975263481613-Snagit-Mac-2026-Version-History
  - https://support.techsmith.com/hc/en-us/articles/37938520706957-Snagit-2025-Version-History
  - https://support.techsmith.com/hc/en-us/articles/37938485483021-Snagit-Mac-2024-Version-History
  - https://support.techsmith.com/hc/en-us/articles/37938496753549-Snagit-Mac-2023-Version-History
  - https://support.techsmith.com/hc/en-us/articles/43415422714125-Snagit-Mac-2022-Version-History
  - https://support.techsmith.com/hc/en-us/articles/43415439595533-Snagit-Mac-2021-Version-History
  - https://support.techsmith.com/hc/en-us/articles/43415462314125-Snagit-Mac-2020-Version-History
  - https://support.techsmith.com/hc/en-us/articles/43415505223437-Snagit-Mac-2019-Version-History

**Support articles**
- https://support.techsmith.com/hc/en-us/articles/41688493993613-What-File-Formats-Does-Snagit-Support
- https://support.techsmith.com/hc/en-us/articles/44545405511821-Supported-File-Types-on-Camtasia-Snagit-for-Mac
- https://support.techsmith.com/hc/en-us/sections/43315610828045-Tips-Workflows-Editor
- https://support.techsmith.com/hc/en-us/sections/200514908-Editor
- https://support.techsmith.com/hc/en-us/articles/45792605008525-Can-I-Set-the-Exact-Position-or-Size-of-Objects-or-Selections-in-Snagit-Editor
- https://support.techsmith.com/hc/en-us/articles/45790491793165-How-to-Set-and-Save-Default-Tool-Properties-in-Snagit
- https://support.techsmith.com/hc/en-us/articles/39980449934093-What-is-Smart-Move
- https://support.techsmith.com/hc/en-us/articles/47858645689485-What-is-Smart-Move
- https://support.techsmith.com/hc/en-us/articles/26140574750861-TechSmith-and-AI
- https://support.techsmith.com/hc/en-us/articles/29029298014733-Handling-SVG-Files-in-Snagit
- https://support.techsmith.com/hc/en-us/articles/4403309311757-Grab-Text-Not-Supported
- https://support.techsmith.com/hc/en-us/articles/360029204112-Turn-Canvas-Snapping-On-or-Off-in-the-Snagit-Editor-Video
- https://support.techsmith.com/hc/en-us/articles/235941148-Image-Captures-Not-Showing-Original-Size-in-Snagit-Editor
- https://support.techsmith.com/hc/en-us/articles/360031483671-Make-an-Image-Transparent-in-Snagit-Video

**Product, feature and changelog pages**
- https://www.techsmith.com/snagit/features/
- https://www.techsmith.com/snagit/features/smart-redact/
- https://www.techsmith.com/snagit/features/background-remover/
- https://www.techsmith.com/snagit/features/crop-image/
- https://www.techsmith.com/snagit/features/image-simplifier/
- https://www.techsmith.com/snagit/features/speech-bubble/
- https://www.techsmith.com/snagit/features/magnify-image/
- https://www.techsmith.com/snagit/features/blur-image/
- https://www.techsmith.com/snagit/whats-new/ (pages 1–6, via ?current_page=2 … 6)
- https://techsmith.com/snagit/whats-new/experimental-feature-improvements
- https://techsmith.com/snagit-2018-press-release.html
- https://www.techsmith.com/template-step-by-step-instructions.html
- https://library.techsmith.com/Snagit/assets/Template/Comparison

**Third-party coverage**
- https://www.videomaker.com/news/techsmith-boosts-snagit-and-camtasia-with-ai-and-screentelligence-powered-workflows/
- https://www.reworked.co/the-wire/snagit-and-camtasia-2025-introduce-ai-and-screentelligence-powered-workflows-for-faster-more-impactful-content-creation/
- https://thejournal.com/articles/2017/10/24/snagit-adds-tools-for-isolating-text-other-image-elements.aspx
