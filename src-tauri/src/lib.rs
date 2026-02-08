use tauri::{Manager, Url, WebviewUrl, WebviewWindowBuilder, webview::PageLoadEvent};

const URL_BAR_JS: &str = r#"
(function() {
    // Don't inject twice
    if (document.getElementById('noto-url-bar')) return;

    const bar = document.createElement('div');
    bar.id = 'noto-url-bar';
    bar.innerHTML = `
        <div id="noto-url-bar-inner" style="
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            height: 36px;
            background: #1a1a2e;
            display: flex;
            align-items: center;
            padding: 0 8px;
            z-index: 2147483647;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            box-shadow: 0 2px 8px rgba(0,0,0,0.3);
            gap: 6px;
        ">
            <button id="noto-back" style="
                background: none; border: none; color: #888; cursor: pointer;
                font-size: 16px; padding: 4px 6px; border-radius: 4px;
            " title="Back">◀</button>
            <button id="noto-forward" style="
                background: none; border: none; color: #888; cursor: pointer;
                font-size: 16px; padding: 4px 6px; border-radius: 4px;
            " title="Forward">▶</button>
            <button id="noto-home" style="
                background: none; border: none; color: #888; cursor: pointer;
                font-size: 16px; padding: 4px 6px; border-radius: 4px;
            " title="Home">🏠</button>
            <input id="noto-url-input" type="text" style="
                flex: 1;
                height: 24px;
                background: #16213e;
                border: 1px solid #333;
                border-radius: 6px;
                color: #e0e0e0;
                padding: 0 10px;
                font-size: 13px;
                outline: none;
                font-family: inherit;
            " />
            <button id="noto-go" style="
                background: #4361ee;
                border: none;
                color: white;
                cursor: pointer;
                font-size: 12px;
                padding: 4px 12px;
                border-radius: 6px;
                font-weight: 600;
            ">Go</button>
            <button id="noto-toggle" style="
                background: none; border: none; color: #666; cursor: pointer;
                font-size: 10px; padding: 4px 6px;
            " title="Hide URL bar">✕</button>
        </div>
    `;
    document.documentElement.appendChild(bar);

    // Push page content down
    document.body.style.marginTop = '36px';

    const input = document.getElementById('noto-url-input');
    input.value = window.location.href;

    // Navigate on Enter
    input.addEventListener('keydown', function(e) {
        if (e.key === 'Enter') {
            let url = input.value.trim();
            if (url && !url.match(/^https?:\/\//)) url = 'https://' + url;
            if (url) window.location.href = url;
        }
    });

    // Focus input on click
    input.addEventListener('focus', function() {
        input.select();
    });

    // Go button
    document.getElementById('noto-go').addEventListener('click', function() {
        let url = input.value.trim();
        if (url && !url.match(/^https?:\/\//)) url = 'https://' + url;
        if (url) window.location.href = url;
    });

    // Back / Forward / Home
    document.getElementById('noto-back').addEventListener('click', function() {
        history.back();
    });
    document.getElementById('noto-forward').addEventListener('click', function() {
        history.forward();
    });
    document.getElementById('noto-home').addEventListener('click', function() {
        window.location.href = 'https://asknoto.app/';
    });

    // Hide bar
    document.getElementById('noto-toggle').addEventListener('click', function() {
        const inner = document.getElementById('noto-url-bar-inner');
        inner.style.display = 'none';
        document.body.style.marginTop = '0';

        // Show a small tab to bring it back
        const tab = document.createElement('div');
        tab.id = 'noto-url-tab';
        tab.style.cssText = 'position:fixed;top:0;right:10px;background:#1a1a2e;color:#4361ee;padding:2px 10px;border-radius:0 0 6px 6px;cursor:pointer;font-size:11px;z-index:2147483647;font-family:-apple-system,sans-serif;opacity:0.7;';
        tab.textContent = '▼ URL Bar';
        tab.addEventListener('click', function() {
            inner.style.display = 'flex';
            document.body.style.marginTop = '36px';
            tab.remove();
        });
        document.documentElement.appendChild(tab);
    });

    // Cmd+L / Ctrl+L to focus URL bar
    document.addEventListener('keydown', function(e) {
        if ((e.metaKey || e.ctrlKey) && e.key === 'l') {
            e.preventDefault();
            const inner = document.getElementById('noto-url-bar-inner');
            if (inner.style.display === 'none') {
                inner.style.display = 'flex';
                document.body.style.marginTop = '36px';
                const tab = document.getElementById('noto-url-tab');
                if (tab) tab.remove();
            }
            input.focus();
            input.select();
        }
    });

    // Hover effects
    ['noto-back','noto-forward','noto-home'].forEach(id => {
        const btn = document.getElementById(id);
        btn.addEventListener('mouseenter', () => btn.style.color = '#4361ee');
        btn.addEventListener('mouseleave', () => btn.style.color = '#888');
    });
    const goBtn = document.getElementById('noto-go');
    goBtn.addEventListener('mouseenter', () => goBtn.style.background = '#3a56d4');
    goBtn.addEventListener('mouseleave', () => goBtn.style.background = '#4361ee');
})();
"#;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let url = WebviewUrl::External(Url::parse("https://asknoto.app/").unwrap());

            let _window = WebviewWindowBuilder::new(app, "main", url)
                .title("Noto")
                .inner_size(1200.0, 900.0)
                .on_navigation(|url| {
                    // Allow all navigation — needed for OAuth/login redirects
                    println!("Navigating to: {}", url);
                    true
                })
                .on_page_load(|webview, payload| {
                    if payload.event() == PageLoadEvent::Finished {
                        // Re-inject the URL bar on every page load
                        let current_url = payload.url().to_string();
                        let js = format!(
                            "{}\ndocument.getElementById('noto-url-input').value = '{}';",
                            URL_BAR_JS,
                            current_url.replace('\'', "\\'")
                        );
                        let _ = webview.eval(&js);
                    }
                })
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
