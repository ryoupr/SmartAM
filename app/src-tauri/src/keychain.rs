use std::process::Command;
use std::sync::LazyLock;

const SERVICE_NAME: &str = "com.smartam.app";

static RE_SAFE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9._@\-]+$").unwrap());

fn validate_input(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() || !RE_SAFE.is_match(value) {
        return Err(format!("Invalid {field}: contains disallowed characters"));
    }
    Ok(())
}

pub fn store_credential(account: &str, key_type: &str, secret: &str) -> Result<(), String> {
    validate_input(account, "account")?;
    validate_input(key_type, "key_type")?;
    let service = format!("{}.{}", SERVICE_NAME, key_type);
    // Delete existing entry (ignore errors)
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", &service, "-a", account])
        .output();
    // Add new entry
    let output = Command::new("security")
        .args(["add-generic-password", "-s", &service, "-a", account, "-w", secret, "-U"])
        .output()
        .map_err(|e| format!("Keychain command failed: {e}"))?;
    if !output.status.success() {
        return Err(format!("Keychain store failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    Ok(())
}

pub fn get_credential(account: &str, key_type: &str) -> Result<String, String> {
    validate_input(account, "account")?;
    validate_input(key_type, "key_type")?;
    let service = format!("{}.{}", SERVICE_NAME, key_type);
    let output = Command::new("security")
        .args(["find-generic-password", "-s", &service, "-a", account, "-w"])
        .output()
        .map_err(|e| format!("Keychain command failed: {e}"))?;
    if !output.status.success() {
        return Err(format!("Credential not found: {}/{}", key_type, account));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn delete_credential(account: &str, key_type: &str) -> Result<(), String> {
    validate_input(account, "account")?;
    validate_input(key_type, "key_type")?;
    let service = format!("{}.{}", SERVICE_NAME, key_type);
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", &service, "-a", account])
        .output();
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ACCOUNT: &str = "test-smartam-ci@test.local";
    const TEST_KEY_TYPE: &str = "test-api-key";

    #[test]
    fn store_and_retrieve_credential() {
        let secret = "test-secret-value-12345";
        // Store
        store_credential(TEST_ACCOUNT, TEST_KEY_TYPE, secret).unwrap();
        // Retrieve
        let retrieved = get_credential(TEST_ACCOUNT, TEST_KEY_TYPE).unwrap();
        assert_eq!(retrieved, secret);
        // Cleanup
        delete_credential(TEST_ACCOUNT, TEST_KEY_TYPE).unwrap();
    }

    #[test]
    fn get_nonexistent_credential_returns_error() {
        let result = get_credential("nonexistent@test.local", "nonexistent-key");
        assert!(result.is_err());
    }

    #[test]
    fn store_overwrites_existing() {
        store_credential(TEST_ACCOUNT, "overwrite-test", "first").unwrap();
        store_credential(TEST_ACCOUNT, "overwrite-test", "second").unwrap();
        let retrieved = get_credential(TEST_ACCOUNT, "overwrite-test").unwrap();
        assert_eq!(retrieved, "second");
        delete_credential(TEST_ACCOUNT, "overwrite-test").unwrap();
    }

    #[test]
    fn delete_nonexistent_succeeds() {
        // delete_credential should not error even if entry doesn't exist
        let result = delete_credential("ghost@test.local", "ghost-key");
        assert!(result.is_ok());
    }
}
