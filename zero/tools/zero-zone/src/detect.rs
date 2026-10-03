use crate::model::Zone;
use std::path::Path;

pub fn detect_zone(path: &Path, home: &Path) -> (Zone, String) {
    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    // Most specific first (workspace before core). The crate tree is wherever the owner says
    // the crates live (INT-267 L2), joined onto this home's repo.
    let core = home.join("0-core");
    let in_crates = zero_core::paths::CRATE_PARENTS
        .iter()
        .any(|p| path.starts_with(core.join(p)));
    if in_crates {
        let rel = path
            .strip_prefix(home.join("0-core"))
            .unwrap_or(path.as_path());
        // Uppercase for critical workspace zone
        (Zone::Workspace, rel.display().to_string().to_uppercase())
    } else if path.starts_with(home.join("0-core")) {
        // Uppercase for critical core zone
        (Zone::Core, "0-CORE".to_string())
    } else if path.starts_with(home.join("1-src")) {
        let rel = path.strip_prefix(home).unwrap_or(path.as_path());
        (Zone::Src, rel.display().to_string())
    } else if path.starts_with(home.join("2-projects")) {
        let rel = path.strip_prefix(home).unwrap_or(path.as_path());
        (Zone::Project, rel.display().to_string())
    } else if path.starts_with(home.join("3-archive")) {
        let rel = path.strip_prefix(home).unwrap_or(path.as_path());
        (Zone::Archive, rel.display().to_string())
    } else {
        // For Scratch zone, replace home path with ~
        let display_path = if path.starts_with(home) {
            path.strip_prefix(home)
                .map(|p| format!("~/{}", p.display()))
                .unwrap_or_else(|_| "~".to_string())
        } else if path == home {
            "~".to_string()
        } else {
            path.display().to_string()
        };
        (Zone::Scratch, display_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// INT-267 L2: a directory inside the crate tree is the Workspace zone. Seen red while the
    /// check named a path the crates left when the tree moved under zero/.
    #[test]
    fn a_crate_directory_is_the_workspace_zone() {
        let home = Path::new("/nonexistent-int267-home");
        let inside = home
            .join("0-core")
            .join(zero_core::paths::CRATE_PARENTS[0])
            .join("alpha/src");
        assert_eq!(detect_zone(&inside, home).0, Zone::Workspace);
    }

    #[test]
    fn the_rest_of_the_repo_is_the_core_zone() {
        let home = Path::new("/nonexistent-int267-home");
        assert_eq!(detect_zone(&home.join("0-core/docs"), home).0, Zone::Core);
    }
}
