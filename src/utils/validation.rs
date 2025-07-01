use crate::error::{AppError, Result};

/// Validates a Kubernetes resource name
pub fn validate_resource_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(AppError::Validation("Resource name cannot be empty".to_string()));
    }
    
    if name.len() > 253 {
        return Err(AppError::Validation("Resource name cannot exceed 253 characters".to_string()));
    }
    
    // Kubernetes name validation: lowercase alphanumeric and dashes, cannot start/end with dash
    let is_valid = name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-');
    
    if !is_valid {
        return Err(AppError::Validation(
            "Resource name must be lowercase alphanumeric characters or '-', and cannot start or end with '-'".to_string()
        ));
    }
    
    Ok(())
}

/// Validates a Kubernetes namespace name
pub fn validate_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty() {
        return Err(AppError::Validation("Namespace cannot be empty".to_string()));
    }
    
    validate_resource_name(namespace)?;
    
    // Additional namespace restrictions
    if namespace == "." || namespace == ".." {
        return Err(AppError::Validation("Namespace cannot be '.' or '..'".to_string()));
    }
    
    Ok(())
}

/// Validates CPU resource specification
pub fn validate_cpu_resource(cpu: &str) -> Result<()> {
    if cpu.is_empty() {
        return Err(AppError::Validation("CPU resource cannot be empty".to_string()));
    }
    
    // Simple validation for CPU format (e.g., "100m", "1", "2.5")
    let is_valid = if cpu.ends_with('m') {
        // Millicores format
        cpu[..cpu.len()-1].parse::<u32>().is_ok()
    } else {
        // Cores format
        cpu.parse::<f64>().is_ok()
    };
    
    if !is_valid {
        return Err(AppError::Validation(
            "Invalid CPU format. Use formats like '100m', '1', or '2.5'".to_string()
        ));
    }
    
    Ok(())
}

/// Validates memory resource specification
pub fn validate_memory_resource(memory: &str) -> Result<()> {
    if memory.is_empty() {
        return Err(AppError::Validation("Memory resource cannot be empty".to_string()));
    }
    
    // Check for valid memory units
    let valid_suffixes = ["Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "K", "M", "G", "T", "P", "E"];
    let has_valid_suffix = valid_suffixes.iter().any(|&suffix| memory.ends_with(suffix));
    
    if !has_valid_suffix {
        return Err(AppError::Validation(
            "Invalid memory format. Use formats like '1Gi', '500Mi', '2G'".to_string()
        ));
    }
    
    // Extract the numeric part and validate
    let numeric_part = valid_suffixes.iter()
        .find_map(|&suffix| memory.strip_suffix(suffix))
        .ok_or_else(|| AppError::Validation("Invalid memory format".to_string()))?;
    
    if numeric_part.parse::<f64>().is_err() {
        return Err(AppError::Validation(
            "Invalid memory format. Numeric part must be a valid number".to_string()
        ));
    }
    
    Ok(())
}

/// Validates storage size specification
pub fn validate_storage_size(size: &str) -> Result<()> {
    validate_memory_resource(size) // Same validation as memory
}

/// Validates container image name
pub fn validate_image_name(image: &str) -> Result<()> {
    if image.is_empty() {
        return Err(AppError::Validation("Container image cannot be empty".to_string()));
    }
    
    // Basic validation - image should contain at least a repository name
    if !image.contains('/') && !image.contains(':') && image.len() < 2 {
        return Err(AppError::Validation(
            "Invalid image format. Use formats like 'nginx', 'nginx:latest', or 'registry/image:tag'".to_string()
        ));
    }
    
    Ok(())
}

/// Validates PostgreSQL database name
pub fn validate_database_name(db_name: &str) -> Result<()> {
    if db_name.is_empty() {
        return Err(AppError::Validation("Database name cannot be empty".to_string()));
    }
    
    if db_name.len() > 63 {
        return Err(AppError::Validation("Database name cannot exceed 63 characters".to_string()));
    }
    
    // PostgreSQL identifier rules: start with letter/underscore, contain letters/digits/underscores
    let first_char = db_name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return Err(AppError::Validation(
            "Database name must start with a letter or underscore".to_string()
        ));
    }
    
    let is_valid = db_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !is_valid {
        return Err(AppError::Validation(
            "Database name can only contain letters, digits, and underscores".to_string()
        ));
    }
    
    Ok(())
}

/// Validates PostgreSQL instance count
pub fn validate_instance_count(instances: i32) -> Result<()> {
    if instances < 1 {
        return Err(AppError::Validation("Instance count must be at least 1".to_string()));
    }
    
    if instances > 10 {
        return Err(AppError::Validation(
            "Instance count cannot exceed 10 for safety reasons".to_string()
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_resource_name() {
        assert!(validate_resource_name("valid-name").is_ok());
        assert!(validate_resource_name("test123").is_ok());
        assert!(validate_resource_name("").is_err());
        assert!(validate_resource_name("-invalid").is_err());
        assert!(validate_resource_name("invalid-").is_err());
        assert!(validate_resource_name("Invalid").is_err());
    }

    #[test]
    fn test_validate_cpu_resource() {
        assert!(validate_cpu_resource("100m").is_ok());
        assert!(validate_cpu_resource("1").is_ok());
        assert!(validate_cpu_resource("2.5").is_ok());
        assert!(validate_cpu_resource("").is_err());
        assert!(validate_cpu_resource("invalid").is_err());
    }

    #[test]
    fn test_validate_memory_resource() {
        assert!(validate_memory_resource("1Gi").is_ok());
        assert!(validate_memory_resource("500Mi").is_ok());
        assert!(validate_memory_resource("2G").is_ok());
        assert!(validate_memory_resource("").is_err());
        assert!(validate_memory_resource("1GB").is_err());
        assert!(validate_memory_resource("invalid").is_err());
    }
}