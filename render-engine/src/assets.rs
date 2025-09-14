use std::collections::HashMap;
use std::sync::LazyLock;
use typst::syntax::package::PackageSpec;

/// String asset entry containing the content and original path
#[derive(Debug, Clone)]
pub struct StringAsset {
    pub content: &'static str,
    pub path: &'static str,
}

/// Binary asset entry containing the content and original path
#[derive(Debug, Clone)]
pub struct BinaryAsset {
    pub content: &'static [u8],
    pub path: &'static str,
}

/// Asset loading result for string assets
#[derive(Debug, Clone)]
pub struct StringAssetResult {
    pub content: &'static str,
    pub path: &'static str,
}

/// Asset loading result for binary assets
#[derive(Debug, Clone)]
pub struct BinaryAssetResult {
    pub content: &'static [u8],
    pub path: &'static str,
}

/// Static string asset registry
static STRING_ASSET_REGISTRY: LazyLock<HashMap<&'static str, StringAsset>> = LazyLock::new(|| {
    let mut assets = HashMap::new();
    
    // Memo loader assets
    assets.insert("memo-loader-main", StringAsset {
        content: include_str!("../memo-loader/main.typ"),
        path: "../memo-loader/main.typ",
    });
    
    // Package assets
    assets.insert("package-typst-toml", StringAsset {
        content: include_str!("../tonguetoquill-usaf-memo/typst.toml"),
        path: "../tonguetoquill-usaf-memo/typst.toml",
    });
    
    assets.insert("package-lib", StringAsset {
        content: include_str!("../tonguetoquill-usaf-memo/src/lib.typ"),
        path: "../tonguetoquill-usaf-memo/src/lib.typ",
    });
    
    assets.insert("package-utils", StringAsset {
        content: include_str!("../tonguetoquill-usaf-memo/src/utils.typ"),
        path: "../tonguetoquill-usaf-memo/src/utils.typ",
    });
    
    assets
});

/// Static binary asset registry
static BINARY_ASSET_REGISTRY: LazyLock<HashMap<&'static str, BinaryAsset>> = LazyLock::new(|| {
    let mut assets = HashMap::new();
    
    // Binary assets
    assets.insert("dod_seal.gif", BinaryAsset {
        content: include_bytes!("../memo-loader/assets/dod_seal.gif"),
        path: "memo-loader/assets/dod_seal.gif",
    });
    
    assets.insert("arial.ttf", BinaryAsset {
        content: include_bytes!("../memo-loader/assets/arial.ttf"),
        path: "memo-loader/assets/arial.ttf",
    });
    
    assets.insert("times.ttf", BinaryAsset {
        content: include_bytes!("../memo-loader/assets/times.ttf"),
        path: "memo-loader/assets/times.ttf",
    });
    
    assets.insert("Times.ttc", BinaryAsset {
        content: include_bytes!("../memo-loader/assets/Times.ttc"),
        path: "memo-loader/assets/Times.ttc",
    });
    
    assets.insert("CopperplateCC-Heavy.otf", BinaryAsset {
        content: include_bytes!("../memo-loader/assets/CopperplateCC-Heavy.otf"),
        path: "memo-loader/assets/CopperplateCC-Heavy.otf",
    });
    
    assets
});

/// Load a string asset by key
pub fn load_string_asset(key: &str) -> Option<StringAssetResult> {
    STRING_ASSET_REGISTRY.get(key).map(|asset| StringAssetResult {
        content: asset.content,
        path: asset.path,
    })
}

/// Load a binary asset by key
pub fn load_binary_asset(key: &str) -> Option<BinaryAssetResult> {
    BINARY_ASSET_REGISTRY.get(key).map(|asset| BinaryAssetResult {
        content: asset.content,
        path: asset.path,
    })
}

/// Resolve package file content by package spec and path
pub fn resolve_package_file(spec: &PackageSpec, path: &str) -> Option<&'static str> {
    if spec.namespace == "preview" && spec.name == "tonguetoquill-usaf-memo" && spec.version.to_string() == "0.1.0" {
        match path {
            "typst.toml" => load_string_asset("package-typst-toml").map(|a| a.content),
            "src/lib.typ" => load_string_asset("package-lib").map(|a| a.content),
            "src/utils.typ" => load_string_asset("package-utils").map(|a: StringAssetResult| a.content),
            _ => None,
        }
    } else {
        None
    }
}

/// Resolve binary asset by path
pub fn resolve_binary_asset(path: &str) -> Option<&'static [u8]> {
    // Find the asset by matching the path against registry entries
    for (_key, asset) in BINARY_ASSET_REGISTRY.iter() {
        if asset.path == path {
            return Some(asset.content);
        }
        // Also check for relative path matches (e.g., "assets/dod_seal.gif" matching "memo-loader/assets/dod_seal.gif")
        if path.ends_with(&asset.path.split('/').last().unwrap_or("")) && 
           asset.path.ends_with(path) {
            return Some(asset.content);
        }
    }
    None
}

/// Get all font assets for font loading
pub fn get_font_assets() -> Vec<BinaryAssetResult> {
    vec!["arial.ttf", "times.ttf", "Times.ttc", "CopperplateCC-Heavy.otf"]
        .into_iter()
        .filter_map(load_binary_asset)
        .collect()
}

/// Get all available string asset keys
pub fn get_string_asset_keys() -> Vec<&'static str> {
    STRING_ASSET_REGISTRY.keys().copied().collect()
}

/// Get all available binary asset keys
pub fn get_binary_asset_keys() -> Vec<&'static str> {
    BINARY_ASSET_REGISTRY.keys().copied().collect()
}

/// Check if a string asset exists
pub fn string_asset_exists(key: &str) -> bool {
    STRING_ASSET_REGISTRY.contains_key(key)
}

/// Check if a binary asset exists
pub fn binary_asset_exists(key: &str) -> bool {
    BINARY_ASSET_REGISTRY.contains_key(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_existing_string_asset() {
        let result = load_string_asset("memo-loader-main");
        assert!(result.is_some());
        
        let asset = result.unwrap();
        assert!(!asset.content.is_empty());
        assert_eq!(asset.path, "../memo-loader/main.typ");
    }
    
    #[test]
    fn test_load_nonexistent_string_asset() {
        let result = load_string_asset("nonexistent");
        assert!(result.is_none());
    }
    
    #[test]
    fn test_load_existing_binary_asset() {
        let result = load_binary_asset("arial.ttf");
        assert!(result.is_some());
        
        let asset = result.unwrap();
        assert!(!asset.content.is_empty());
        assert_eq!(asset.path, "memo-loader/assets/arial.ttf");
    }
    
    #[test]
    fn test_string_asset_exists() {
        assert!(string_asset_exists("memo-loader-main"));
        assert!(string_asset_exists("package-lib"));
        assert!(!string_asset_exists("nonexistent"));
    }
    
    #[test]
    fn test_binary_asset_exists() {
        assert!(binary_asset_exists("arial.ttf"));
        assert!(binary_asset_exists("dod_seal.gif"));
        assert!(!binary_asset_exists("nonexistent"));
    }
    
    #[test]
    fn test_get_string_asset_keys() {
        let keys = get_string_asset_keys();
        assert!(keys.contains(&"memo-loader-main"));
        assert!(keys.contains(&"package-typst-toml"));
        assert!(keys.contains(&"package-lib"));
        assert!(keys.contains(&"package-utils"));
    }
    
    #[test]
    fn test_get_binary_asset_keys() {
        let keys = get_binary_asset_keys();
        assert!(keys.contains(&"arial.ttf"));
        assert!(keys.contains(&"times.ttf"));
        assert!(keys.contains(&"Times.ttc"));
        assert!(keys.contains(&"CopperplateCC-Heavy.otf"));
        assert!(keys.contains(&"dod_seal.gif"));
    }
    
    #[test]
    fn test_all_string_assets_loadable() {
        let keys = get_string_asset_keys();
        for key in keys {
            let result = load_string_asset(key);
            assert!(result.is_some(), "String asset '{}' should be loadable", key);
            
            let asset = result.unwrap();
            assert!(!asset.content.is_empty(), "String asset '{}' should have content", key);
            assert!(!asset.path.is_empty(), "String asset '{}' should have a path", key);
        }
    }
    
    #[test]
    fn test_all_binary_assets_loadable() {
        let keys = get_binary_asset_keys();
        for key in keys {
            let result = load_binary_asset(key);
            assert!(result.is_some(), "Binary asset '{}' should be loadable", key);
            
            let asset = result.unwrap();
            assert!(!asset.content.is_empty(), "Binary asset '{}' should have content", key);
            assert!(!asset.path.is_empty(), "Binary asset '{}' should have a path", key);
        }
    }
    
    #[test]
    fn test_get_font_assets() {
        let fonts = get_font_assets();
        assert_eq!(fonts.len(), 4);
        
        // Check that all expected fonts are present
        let font_names: Vec<&str> = fonts.iter().map(|f| {
            f.path.split('/').last().unwrap()
        }).collect();
        
        assert!(font_names.contains(&"arial.ttf"));
        assert!(font_names.contains(&"times.ttf"));
        assert!(font_names.contains(&"Times.ttc"));
        assert!(font_names.contains(&"CopperplateCC-Heavy.otf"));
    }
}