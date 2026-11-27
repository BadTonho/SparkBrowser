use crate::sync::Mutex;
use std::collections::HashMap;

struct Document {
    generation: String,
    ready: bool,
    target: String,
}

/// Orders startup navigation and user requests while native handlers are installing.
#[derive(Default)]
pub(crate) struct ContentBootstrap(Mutex<HashMap<String, Document>>);

impl ContentBootstrap {
    pub(crate) fn begin(&self, label: &str, generation: &str, target: &str) -> Result<(), String> {
        self.0.lock().map_err(|e| e.to_string())?.insert(
            label.into(),
            Document {
                generation: generation.into(),
                ready: false,
                target: target.into(),
            },
        );
        Ok(())
    }

    // Dispatch is synchronous and must not reenter this registry. Holding the guard
    // orders native dispatch against requests arriving on another thread.
    pub(crate) fn navigate(
        &self,
        label: &str,
        target: &str,
        dispatch: impl FnOnce(&str) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut documents = self.0.lock().map_err(|e| e.to_string())?;
        if let Some(document) = documents.get_mut(label) {
            if !document.ready {
                document.target = target.into();
                return Ok(());
            }
        }
        dispatch(target)
    }

    pub(crate) fn finish(
        &self,
        label: &str,
        generation: &str,
        dispatch: impl FnOnce(&str) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut documents = self.0.lock().map_err(|e| e.to_string())?;
        let Some(document) = documents.get_mut(label) else {
            return Ok(());
        };
        if document.generation != generation || document.ready {
            return Ok(());
        }
        dispatch(&document.target)?;
        document.ready = true;
        Ok(())
    }

    pub(crate) fn retire(&self, label: &str) -> Result<(), String> {
        self.0.lock().map_err(|e| e.to_string())?.remove(label);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn early_requests_replace_initial_navigation_and_handlers_finish_before_dispatch() {
        let bootstrap = ContentBootstrap::default();
        bootstrap.begin("tab", "generation", "new-tab").unwrap();
        for target in ["first-user-request", "latest-user-request"] {
            bootstrap
                .navigate("tab", target, |_| panic!("handlers are not ready"))
                .unwrap();
        }
        bootstrap
            .finish("tab", "generation", |target| {
                assert_eq!(target, "latest-user-request");
                Ok(())
            })
            .unwrap();
        bootstrap
            .finish("tab", "generation", |_| {
                panic!("startup must dispatch once")
            })
            .unwrap();
        bootstrap
            .navigate("tab", "next", |target| {
                assert_eq!(target, "next");
                Ok(())
            })
            .unwrap();
    }
    #[test]
    fn retired_generations_cannot_navigate_a_recreated_document() {
        let bootstrap = ContentBootstrap::default();
        bootstrap.begin("tab", "old", "old-target").unwrap();
        bootstrap.retire("tab").unwrap();
        bootstrap
            .finish("tab", "old", |_| panic!("document was closed"))
            .unwrap();
        bootstrap.begin("tab", "new", "new-target").unwrap();
        bootstrap
            .finish("tab", "old", |_| panic!("generation was replaced"))
            .unwrap();
        assert!(bootstrap
            .finish("tab", "new", |_| Err("injected failure".into()))
            .is_err());
        bootstrap
            .finish("tab", "new", |target| {
                assert_eq!(target, "new-target");
                Ok(())
            })
            .unwrap();
    }
}
