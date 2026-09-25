//! Deployment roles are startup policy, never user permissions or DB defaults.
pub static PERSONA_RUNTIME_LOCAL: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

pub static FEDERATION_HTTP_ISOLATED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub static PERSONA_HTTP_ISOLATED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
pub static PERSONA_WORKER: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRole {
    Web,
    FederationWorker,
    PersonaWorker,
    All,
}

impl RuntimeRole {
    pub fn from_env() -> anyhow::Result<Self> {
        let executable = std::env::args_os().next();
        match executable
            .as_deref()
            .and_then(|path| std::path::Path::new(path).file_name())
            .and_then(|name| name.to_str())
        {
            Some("myriad-federation-worker") => return Ok(Self::FederationWorker),
            Some("myriad-persona-worker") => return Ok(Self::PersonaWorker),
            _ => {}
        }
        match std::env::var("MYRIAD_PROCESS_ROLE") {
            Ok(value) => Self::parse(
                &value,
                crate::config::AppConfig::is_production_environment(),
                combined_runtime_allowed(),
            ),
            Err(std::env::VarError::NotPresent) => Self::missing_role(),
            Err(error) => Err(error.into()),
        }
    }

    fn missing_role() -> anyhow::Result<Self> {
        anyhow::bail!(
            "MYRIAD_PROCESS_ROLE is required: set web for the normal topology, or all only \
             with MYRIAD_ALLOW_COMBINED_RUNTIME=true in a constrained single-container deployment"
        )
    }

    fn parse(value: &str, production: bool, allow_combined: bool) -> anyhow::Result<Self> {
        match value {
            "web" => Ok(Self::Web),
            "federation-worker" => Ok(Self::FederationWorker),
            "persona-worker" => Ok(Self::PersonaWorker),
            "all" if !production || allow_combined => Ok(Self::All),
            "all" => anyhow::bail!(
                "MYRIAD_PROCESS_ROLE=all requires MYRIAD_ALLOW_COMBINED_RUNTIME=true in production"
            ),
            _ => anyhow::bail!(
                "MYRIAD_PROCESS_ROLE must be web, federation-worker, persona-worker, or explicitly enabled all"
            ),
        }
    }
}

fn combined_runtime_allowed() -> bool {
    matches!(
        std::env::var("MYRIAD_ALLOW_COMBINED_RUNTIME").as_deref(),
        Ok("1" | "true" | "TRUE" | "yes" | "on")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_never_silently_combines_roles() {
        assert!(RuntimeRole::missing_role().is_err());
        assert_eq!(
            RuntimeRole::parse("web", true, false).unwrap(),
            RuntimeRole::Web
        );
        assert_eq!(
            RuntimeRole::parse("federation-worker", true, false).unwrap(),
            RuntimeRole::FederationWorker
        );
        assert_eq!(
            RuntimeRole::parse("persona-worker", true, false).unwrap(),
            RuntimeRole::PersonaWorker
        );
        assert!(RuntimeRole::parse("all", true, false).is_err());
        assert_eq!(
            RuntimeRole::parse("all", true, true).unwrap(),
            RuntimeRole::All
        );
        assert!(RuntimeRole::parse("federaton-worker", false, false).is_err());
        assert_eq!(
            RuntimeRole::parse("all", false, false).unwrap(),
            RuntimeRole::All
        );
    }
}
