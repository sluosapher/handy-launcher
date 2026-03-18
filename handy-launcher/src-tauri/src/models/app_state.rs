use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagedOllamaState {
    pub port: Option<u16>,
    pub generation: u64,
    pub restart_count: u8,
}

#[derive(Debug)]
struct ManagedOllamaInner {
    port: Option<u16>,
    generation: u64,
    restart_count: u8,
}

#[derive(Debug, Default)]
pub struct AppState {
    managed_ollama: Mutex<ManagedOllamaInner>,
}

impl Default for ManagedOllamaInner {
    fn default() -> Self {
        Self {
            port: None,
            generation: 0,
            restart_count: 0,
        }
    }
}

impl AppState {
    pub fn mark_ollama_started(&self, port: u16) -> ManagedOllamaState {
        let mut state = self
            .managed_ollama
            .lock()
            .expect("managed ollama state lock poisoned");
        state.port = Some(port);
        state.generation += 1;
        state.restart_count = 0;
        ManagedOllamaState {
            port: state.port,
            generation: state.generation,
            restart_count: state.restart_count,
        }
    }

    pub fn clear_managed_ollama(&self) -> ManagedOllamaState {
        let mut state = self
            .managed_ollama
            .lock()
            .expect("managed ollama state lock poisoned");
        state.port = None;
        state.generation += 1;
        state.restart_count = 0;
        ManagedOllamaState {
            port: state.port,
            generation: state.generation,
            restart_count: state.restart_count,
        }
    }

    pub fn managed_ollama(&self) -> ManagedOllamaState {
        let state = self
            .managed_ollama
            .lock()
            .expect("managed ollama state lock poisoned");
        ManagedOllamaState {
            port: state.port,
            generation: state.generation,
            restart_count: state.restart_count,
        }
    }

    pub fn should_restart_ollama(&self, port: u16, generation: u64, max_restarts: u8) -> bool {
        let mut state = self
            .managed_ollama
            .lock()
            .expect("managed ollama state lock poisoned");

        if state.port != Some(port) || state.generation != generation {
            return false;
        }

        if state.restart_count >= max_restarts {
            return false;
        }

        state.restart_count += 1;
        true
    }

    pub fn mark_restart_succeeded(&self, port: u16, generation: u64) {
        let mut state = self
            .managed_ollama
            .lock()
            .expect("managed ollama state lock poisoned");

        if state.port == Some(port) && state.generation == generation {
            state.restart_count = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;

    #[test]
    fn mark_ollama_started_tracks_managed_port_and_resets_restart_count() {
        let state = AppState::default();
        let started = state.mark_ollama_started(63452);

        assert_eq!(started.port, Some(63452));
        assert_eq!(started.generation, 1);
        assert_eq!(started.restart_count, 0);
    }

    #[test]
    fn clear_managed_ollama_increments_generation_and_clears_port() {
        let state = AppState::default();
        state.mark_ollama_started(63452);

        let cleared = state.clear_managed_ollama();

        assert_eq!(cleared.port, None);
        assert_eq!(cleared.generation, 2);
        assert_eq!(cleared.restart_count, 0);
    }

    #[test]
    fn should_restart_only_for_current_generation_and_until_limit() {
        let state = AppState::default();
        let managed = state.mark_ollama_started(63452);

        assert!(state.should_restart_ollama(63452, managed.generation, 2));
        assert!(state.should_restart_ollama(63452, managed.generation, 2));
        assert!(!state.should_restart_ollama(63452, managed.generation, 2));
        assert!(!state.should_restart_ollama(63453, managed.generation, 2));
        assert!(!state.should_restart_ollama(63452, managed.generation + 1, 2));
    }

    #[test]
    fn mark_restart_succeeded_resets_restart_budget_for_current_generation() {
        let state = AppState::default();
        let managed = state.mark_ollama_started(63452);
        assert!(state.should_restart_ollama(63452, managed.generation, 1));

        state.mark_restart_succeeded(63452, managed.generation);

        assert!(state.should_restart_ollama(63452, managed.generation, 1));
    }
}
