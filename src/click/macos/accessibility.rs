use objc2_application_services::{AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt};
use objc2_core_foundation::{CFBoolean, CFDictionary};
use std::{
    thread,
    time::{Duration, Instant},
};
use tracing::{error, info, warn};

const AX_POLL_INTERVAL: Duration = Duration::from_millis(250);
const AX_POLL_TIMEOUT: Duration = Duration::from_secs(30);

fn is_trusted() -> bool {
    let keys = &[unsafe { kAXTrustedCheckOptionPrompt }];
    let values = &[CFBoolean::new(true)];
    let options = CFDictionary::from_slices(keys, values);
    unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) }
}

fn prompt_trust_dialog() {
    let key = unsafe { kAXTrustedCheckOptionPrompt };
    let value = CFBoolean::new(true);
    let options = CFDictionary::from_slices(&[key], &[value]);
    unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) };
}

pub fn ensure_trust() {
    if is_trusted() {
        info!("Accessibility permission already granted");
        return;
    }

    info!("Accessibility permission not granted; prompting user to enable it in System Settings.");

    prompt_trust_dialog();

    let start = Instant::now();
    loop {
        if is_trusted() {
            info!(
                "Accessibility permission granted after {:.1?}",
                start.elapsed()
            );
            return;
        }

        let elapsed = start.elapsed();
        if elapsed >= AX_POLL_TIMEOUT {
            break;
        }

        thread::sleep(AX_POLL_INTERVAL);
    }

    warn!(
        "Accessibility permission not granted after {:?}.",
        AX_POLL_TIMEOUT
    );
    error!(
        "Click does not have accessibility permissions. Please enable it manually at: \
         System Settings → Privacy & Security → Accessibility."
    );

    std::process::exit(1);
}
