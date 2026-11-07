pub mod adblock_engine;
pub mod auto_skip;

// The navigation policy is test-only until the platform event adapters can
// enforce its lifecycle without gaps in WebView2 and WebKitGTK.
#[cfg(test)]
mod network_policy;
