//! Web commands: webget, webserver, and simple HTTP-like intent recording.

use crate::commands::helpers as h;
use crate::commands::registry::{register, CommandOutcome};
use crate::obj::{ElementTag, ObjectTag};

fn intent(command: &str, detail: &str) {
    if detail.is_empty() {
        crate::util::debug_log(&format!("[{}]", command));
    } else {
        crate::util::debug_log(&format!("[{}] {}", command, detail));
    }
}

pub fn register_all() {
    register("webget", "webget [<url>] (post:<data>) (headers:<map>) (save:<name>)", |entry, ctx, q| {
        let url = h::arg_tag(entry, 0, ctx);
        intent("webget", &url);
        let save = h::prefixed(entry, "save", ctx).unwrap_or_else(|| "value".to_string());
        // Network access is disabled by default in this build; record an empty result.
        if crate::engine::config::ALLOW_NETWORK.load(std::sync::atomic::Ordering::Relaxed) {
            // Actual network I/O is delegated to the host; record intent only.
        }
        h::set_def(q, &save, ObjectTag::new(ElementTag::new("")));
        Ok(CommandOutcome::Continue)
    });

    register("webserver", "webserver [<action>] (port:<port>) (id:<id>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let port = h::prefixed(entry, "port", ctx).unwrap_or_default();
        intent("webserver", &format!("{} port:{}", action, port));
        Ok(CommandOutcome::Continue)
    });

    register("http", "http [<method>] [<url>] (body:<body>) (save:<name>)", |entry, ctx, q| {
        let method = h::arg_tag(entry, 0, ctx);
        let url = h::arg_tag(entry, 1, ctx);
        intent("http", &format!("{} {}", method, url));
        let save = h::prefixed(entry, "save", ctx).unwrap_or_else(|| "value".to_string());
        h::set_def(q, &save, ObjectTag::new(ElementTag::new("")));
        Ok(CommandOutcome::Continue)
    });
}
