//! Compartir del sistema (SHR-037): the Windows Share window for the files prepared
//! (Nearby sharing, mail and the apps that take them). XcreenShot only hands the files
//! over; what happens next is up to Windows or the other app.

#[cfg(windows)]
pub fn show(app: &tauri::AppHandle, files: Vec<std::path::PathBuf>, title: String) -> Result<(), String> {
    use tauri::Manager;
    use windows::core::HSTRING;
    use windows::Storage::{IStorageItem, StorageFile};

    // The files first, away from the window's thread: getting them waits. They cross
    // to that thread as agile references.
    let mut items: Vec<windows::core::AgileReference<IStorageItem>> = Vec::new();
    for f in &files {
        // Windows wants backslashes only.
        let path = f.to_string_lossy().replace('/', "\\");
        let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path.as_str()))
            .and_then(|op| op.join())
            .map_err(|e| e.message().to_string())?;
        let item: IStorageItem =
            windows::core::Interface::cast(&file).map_err(|e| e.message().to_string())?;
        items.push(windows::core::AgileReference::new(&item).map_err(|e| e.message().to_string())?);
    }
    let window = app.get_webview_window("main").ok_or("no window")?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(on_window(hwnd, items, title).map_err(|e| e.message().to_string()));
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}

#[cfg(windows)]
fn on_window(
    hwnd: isize,
    items: Vec<windows::core::AgileReference<windows::Storage::IStorageItem>>,
    title: String,
) -> windows::core::Result<()> {
    use std::cell::RefCell;
    use windows::core::HSTRING;
    use windows::ApplicationModel::DataTransfer::{DataRequestedEventArgs, DataTransferManager};
    use windows::Foundation::TypedEventHandler;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::IDataTransferManagerInterop;
    use windows_collections::IIterable;

    thread_local! {
        /// The window's manager and the handler of the last share, replaced each time.
        static LAST: RefCell<Option<(DataTransferManager, i64)>> = const { RefCell::new(None) };
    }
    let items = std::sync::Arc::new(items);

    let interop = windows::core::factory::<DataTransferManager, IDataTransferManagerInterop>()?;
    let hwnd = HWND(hwnd as *mut core::ffi::c_void);
    // SAFETY: the main window's handle, on its own thread.
    let manager: DataTransferManager = unsafe { interop.GetForWindow(hwnd)? };
    if let Some((old, token)) = LAST.with(|l| l.borrow_mut().take()) {
        let _ = old.RemoveDataRequested(token);
    }
    let token = manager.DataRequested(&TypedEventHandler::new(
        move |_, args: windows::core::Ref<DataRequestedEventArgs>| {
            if let Some(args) = args.as_ref() {
                let data = args.Request()?.Data()?;
                data.Properties()?.SetTitle(&HSTRING::from(title.as_str()))?;
                let list: Vec<Option<windows::Storage::IStorageItem>> =
                    items.iter().map(|r| r.resolve().ok()).collect();
                data.SetStorageItemsReadOnly(&IIterable::from(list))?;
            }
            Ok(())
        },
    ))?;
    LAST.with(|l| *l.borrow_mut() = Some((manager, token)));
    // SAFETY: as above.
    unsafe { interop.ShowShareUIForWindow(hwnd) }
}

#[cfg(not(windows))]
pub fn show(_app: &tauri::AppHandle, _files: Vec<std::path::PathBuf>, _title: String) -> Result<(), String> {
    Err("not yet on this system".into())
}
