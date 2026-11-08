//! `morc annoy`: read and nudge the annoyance vector. Split out of cli/mod.rs.

use super::*;

/// The annoyance vector: `morc annoy` shows it; the rest change it (and log why). Run when the user complains.
///   morc annoy set <param> <value> [--topic T] [--why "..."]     absolute
///   morc annoy nudge <param> <factor> [--topic T] [--why "..."]  multiply, e.g. size_bias 0.85 = "a bit smaller"
///   morc annoy quiet <HH:MM> <HH:MM> | off [--why ...]            no auto pushes in that window
///   morc annoy mute-app <exe> | unmute-app <exe> [--why ...]     no auto pushes while that app is in front
///   morc annoy mute-topic <topic> | unmute-topic <topic>
pub(super) fn annoy(rest: &[String]) -> i32 {
    use crate::prefs;
    let why = flag(rest, "--why").unwrap_or_default();
    let topic = flag(rest, "--topic");
    let r: Result<String, String> = (|| match rest.first().map(|s| s.as_str()) {
        None | Some("show") => Ok(prefs::describe().to_string()),
        Some("set") | Some("nudge") => {
            let (name, val) = (rest.get(1).ok_or("annoy set|nudge <param> <value>")?, rest.get(2).ok_or("missing value")?.parse::<f64>().map_err(|e| e.to_string())?);
            let v = if rest[0] == "set" { prefs::set(topic.as_deref(), name, val, "user", &why)? } else { prefs::nudge(topic.as_deref(), name, val, "user", &why)? };
            Ok(format!("{{\"ok\":true,\"param\":\"{name}\",\"value\":{v}}}"))
        }
        Some("quiet") => {
            let range = match rest.get(1).map(|s| s.as_str()) { Some("off") => None, Some(a) => Some((a.to_string(), rest.get(2).ok_or("quiet <HH:MM> <HH:MM>")?.clone())), None => return Err("quiet <HH:MM> <HH:MM> | off".into()) };
            prefs::set_quiet(range, &why)?;
            Ok("{\"ok\":true}".into())
        }
        Some("theme") => {
            prefs::set_theme(rest.get(1).ok_or("annoy theme <coral|ocean|violet|rose|amber|emerald|lime|sky|fuchsia|teal|auto>")?, &why)?;
            Ok("{\"ok\":true}".into())
        }
        Some(c @ ("mute-app" | "unmute-app")) => {
            prefs::mute_app(rest.get(1).ok_or("need an exe name, e.g. chrome.exe")?, c == "mute-app", &why)?;
            Ok("{\"ok\":true}".into())
        }
        Some(c @ ("mute-topic" | "unmute-topic")) => {
            let t = rest.get(1).ok_or("need a topic")?;
            prefs::set(Some(t), "mute", if c == "mute-topic" { 1.0 } else { 0.0 }, "user", &why)?;
            Ok("{\"ok\":true}".into())
        }
        Some(o) => Err(format!("unknown annoy command '{o}'")),
    })();
    match r {
        Ok(s) => { println!("{s}"); 0 }
        Err(e) => { eprintln!("morc annoy: {e}"); 2 }
    }
}

