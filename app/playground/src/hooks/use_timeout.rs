use std::time::Duration;

use dioxus::{
    hooks::{use_resource, Resource},
    signals::{ReadableExt, Signal},
};

/// A hook that runs a callback after a delay, when the signal is triggered.
pub fn use_timeout<FnCallback>(
    duration: Duration,
    signal_triggered: Signal<bool>,
    mut fn_callback: FnCallback,
) -> Resource<()>
where
    FnCallback: Copy + FnMut() + 'static,
{
    use_resource(move || async move {
        let triggered = *signal_triggered.read();
        if !triggered {
            return;
        }

        #[cfg(target_family = "wasm")]
        sleep(duration.as_millis() as u32).await;

        // suppresses unused variable warning.
        #[cfg(not(target_family = "wasm"))]
        let _duration = duration;

        fn_callback();
    })
}

#[cfg(target_family = "wasm")]
async fn sleep(duration_ms: u32) {
    gloo_timers::future::TimeoutFuture::new(duration_ms).await;
}

// If we ever support desktop or need this run on the server, we can use this.
//
// ```rust,ignore
// #[cfg(not(target_family = "wasm"))]
// async fn sleep(duration_ms: u64) {
//     tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
// }
// ```
