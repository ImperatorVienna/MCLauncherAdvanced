// Native toolkit identity only; no presentation or navigation policy.
pub fn initialize_identity(identifier: &str) {
    #[cfg(target_os = "linux")]
    // GDK uses the program name for xdg_toplevel.app_id in this GTK backend.
    // Set it before GTK initialization, independently of the executable filename.
    glib::set_prgname(Some(identifier));
    #[cfg(not(target_os = "linux"))]
    let _ = identifier;
}

// GTK's clipboard owns the text for the running desktop session (Wayland/X11).
// Must be called on the GTK main thread. Other hosts retain WebView copying.
pub fn copy_clipboard_text(text:&str)->bool {
    #[cfg(target_os="linux")]
    {
        let Some(display)=gtk::gdk::Display::default() else {return false;};
        let Some(clipboard)=gtk::Clipboard::default(&display) else {return false;};
        clipboard.set_text(text);
        clipboard.store();
        true
    }
    #[cfg(not(target_os="linux"))]
    {let _=text;false}
}
