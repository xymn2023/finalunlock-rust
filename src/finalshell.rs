use anyhow::Result;
use md5::{Digest, Md5};
use sha3::Keccak384;

use crate::models::FinalShellVersion;

/// FinalShell版本枚举
#[derive(Debug, Clone)]
pub enum FinalShellVersionType {
    Legacy,      // < 3.9.6
    V396Plus,    // ≥ 3.9.6
    V45,         // 4.5
    V46,         // 4.6
}

/// 激活码类型
#[derive(Debug, Clone)]
pub enum LicenseType {
    Advanced,    // 高级版
    Professional, // 专业版
}

/// 激活码结果
#[derive(Debug, Clone)]
pub struct ActivationResult {
    pub version_type: FinalShellVersionType,
    pub advanced_code: String,
    pub professional_code: String,
    pub version_name: String,
}

/// FinalShell激活码生成器
pub struct ActivationCodeGenerator;

impl ActivationCodeGenerator {
    /// 根据机器码生成所有版本的激活码
    pub fn generate_all(machine_code: &str) -> Result<Vec<ActivationResult>> {
        let mut results = Vec::new();
        
        // 生成所有版本的激活码
        results.push(Self::generate_legacy(machine_code)?);
        results.push(Self::generate_v396_plus(machine_code)?);
        results.push(Self::generate_v45(machine_code)?);
        results.push(Self::generate_v46(machine_code)?);
        
        Ok(results)
    }
    
    /// 根据机器码生成默认版本激活码 (用于向后兼容)
    pub fn generate(machine_code: &str) -> Result<(String, FinalShellVersion)> {
        let version = FinalShellVersion::detect_version(machine_code);
        
        let activation_code = match version.version.as_str() {
            "< 3.9.6" => {
                let result = Self::generate_legacy(machine_code)?;
                result.professional_code // 默认返回专业版
            },
            "≥ 3.9.6" => {
                let result = Self::generate_v396_plus(machine_code)?;
                result.professional_code
            },
            "4.5" => {
                let result = Self::generate_v45(machine_code)?;
                result.professional_code
            },
            "4.6+" => {
                let result = Self::generate_v46(machine_code)?;
                result.professional_code
            },
            _ => {
                let result = Self::generate_v396_plus(machine_code)?;
                result.professional_code
            }
        };

        Ok((activation_code, version))
    }

    /// 计算MD5哈希
    fn calc_md5(data: &str) -> Result<String> {
        let mut hasher = Md5::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// 计算Keccak384哈希
    fn calc_keccak384(data: &str) -> Result<String> {
        let mut hasher = Keccak384::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// 生成3.9.6以前版本的激活码
    fn generate_legacy(machine_code: &str) -> Result<ActivationResult> {
        // 🟡 高级版: MD5(61305{machine_id}8552)[8:24]
        let advanced_hash = Self::calc_md5(&format!("61305{}8552", machine_code))?;
        let advanced_code = advanced_hash[8..24].to_uppercase();
        
        // 🟢 专业版: MD5(2356{machine_id}13593)[8:24]
        let professional_hash = Self::calc_md5(&format!("2356{}13593", machine_code))?;
        let professional_code = professional_hash[8..24].to_uppercase();

        Ok(ActivationResult {
            version_type: FinalShellVersionType::Legacy,
            advanced_code,
            professional_code,
            version_name: "FinalShell < 3.9.6".to_string(),
        })
    }

    /// 生成3.9.6及以后版本的激活码
    fn generate_v396_plus(machine_code: &str) -> Result<ActivationResult> {
        // 🟡 高级版: Keccak384({machine_id}hSf(78cvVlS5E)[12:28]
        let advanced_hash = Self::calc_keccak384(&format!("{}hSf(78cvVlS5E", machine_code))?;
        let advanced_code = advanced_hash[12..28].to_uppercase();
        
        // 🟢 专业版: Keccak384({machine_id}FF3Go(*Xvbb5s2)[12:28]
        let professional_hash = Self::calc_keccak384(&format!("{}FF3Go(*Xvbb5s2", machine_code))?;
        let professional_code = professional_hash[12..28].to_uppercase();

        Ok(ActivationResult {
            version_type: FinalShellVersionType::V396Plus,
            advanced_code,
            professional_code,
            version_name: "FinalShell ≥ 3.9.6".to_string(),
        })
    }

    /// 生成4.5版本的激活码
    fn generate_v45(machine_code: &str) -> Result<ActivationResult> {
        // 🟡 高级版: Keccak384({machine_id}wcegS3gzA$)[12:28]
        let advanced_hash = Self::calc_keccak384(&format!("{}wcegS3gzA$", machine_code))?;
        let advanced_code = advanced_hash[12..28].to_uppercase();
        
        // 🟢 专业版: Keccak384({machine_id}b(xxkHn%z);x)[12:28]
        let professional_hash = Self::calc_keccak384(&format!("{}b(xxkHn%z);x", machine_code))?;
        let professional_code = professional_hash[12..28].to_uppercase();

        Ok(ActivationResult {
            version_type: FinalShellVersionType::V45,
            advanced_code,
            professional_code,
            version_name: "FinalShell 4.5".to_string(),
        })
    }

    /// 生成4.6版本的激活码
    fn generate_v46(machine_code: &str) -> Result<ActivationResult> {
        // 🟡 高级版: Keccak384({machine_id}csSf5*xlkgYSX,y)[12:28]
        let advanced_hash = Self::calc_keccak384(&format!("{}csSf5*xlkgYSX,y", machine_code))?;
        let advanced_code = advanced_hash[12..28].to_uppercase();
        
        // 🟢 专业版: Keccak384({machine_id}Scfg*ZkvJZc,s,Y)[12:28]
        let professional_hash = Self::calc_keccak384(&format!("{}Scfg*ZkvJZc,s,Y", machine_code))?;
        let professional_code = professional_hash[12..28].to_uppercase();

        Ok(ActivationResult {
            version_type: FinalShellVersionType::V46,
            advanced_code,
            professional_code,
            version_name: "FinalShell 4.6".to_string(),
        })
    }

    /// 验证机器码格式
    pub fn validate_machine_code(machine_code: &str) -> bool {
        if machine_code.is_empty() || machine_code.len() < 8 {
            return false;
        }

        // 基本格式检查
        let trimmed = machine_code.trim();
        
        // 检查是否包含有效字符（FinalShell机器码可能包含@符号）
        trimmed.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '@'
        })
    }

    /// 清理机器码格式
    pub fn clean_machine_code(machine_code: &str) -> String {
        machine_code
            .trim()
            .replace(' ', "")
            .replace('\n', "")
            .replace('\r', "")
            .replace('\t', "")
    }

    /// 检测FinalShell版本信息
    pub fn detect_version_info(machine_code: &str) -> String {
        let version = FinalShellVersion::detect_version(machine_code);
        match version.version.as_str() {
            "< 3.9.6" => "FinalShell 3.9.6 以前版本".to_string(),
            "≥ 3.9.6" => "FinalShell 3.9.6 及以后版本".to_string(),
            "4.5" => "FinalShell 4.5 版本".to_string(),
            "4.6+" => "FinalShell 4.6 及以后版本".to_string(),
            _ => "FinalShell 通用版本".to_string(),
        }
    }

    /// 格式化所有版本的激活码结果
    pub fn format_all_codes(machine_code: &str) -> Result<String> {
        let results = Self::generate_all(machine_code)?;
        
        let mut output = String::new();
        
        // 添加美化的头部
        output.push_str("═══════════════════════════════════════\n");
        output.push_str("🎉        FinalShell 激活码生成器        🎉\n");
        output.push_str("═══════════════════════════════════════\n\n");
        
        output.push_str(&format!("🔑 输入机器码: `{}`\n", machine_code));
        output.push_str(&format!("📅 生成时间: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        output.push_str("🎯 生成结果:\n\n");
        
        for (index, result) in results.iter().enumerate() {
            let version_icon = match index {
                0 => "🔹", // < 3.9.6
                1 => "🔸", // ≥ 3.9.6
                2 => "🔷", // 4.5
                3 => "🔶", // 4.6
                _ => "📌",
            };
            
            output.push_str(&format!(
                "{} {}\n\
                 ┣━ 🟡 高级版: `{}`\n\
                 ┗━ 🟢 专业版: `{}`\n\n",
                version_icon,
                result.version_name,
                result.advanced_code,
                result.professional_code
            ));
        }
        
        output.push_str("═══════════════════════════════════════\n");
        output.push_str("💡 提示: 欢迎使用 🟢 激活码生成工具\n");
        output.push_str("🛡️ 请合理使用 滥用必究\n");
        output.push_str("═══════════════════════════════════════\n");
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_machine_code() {
        assert!(ActivationCodeGenerator::validate_machine_code("ABC123DEF456"));
        assert!(ActivationCodeGenerator::validate_machine_code("abc-123-def"));
        assert!(!ActivationCodeGenerator::validate_machine_code(""));
        assert!(!ActivationCodeGenerator::validate_machine_code("123"));
        assert!(!ActivationCodeGenerator::validate_machine_code("ABC@123"));
    }

    #[test]
    fn test_clean_machine_code() {
        let input = " ABC 123\nDEF\t456 ";
        let expected = "ABC123DEF456";
        assert_eq!(ActivationCodeGenerator::clean_machine_code(input), expected);
    }

    #[test]
    fn test_generate_activation_code() {
        let machine_code = "ABC123DEF456";
        let result = ActivationCodeGenerator::generate(machine_code);
        assert!(result.is_ok());
        
        let (activation_code, version) = result.unwrap();
        assert!(!activation_code.is_empty());
        assert_eq!(activation_code.len(), 16); // 激活码长度应该是16位
    }

    #[test]
    fn test_generate_all_codes() {
        let machine_code = "ABC123DEF456";
        let result = ActivationCodeGenerator::generate_all(machine_code);
        assert!(result.is_ok());
        
        let results = result.unwrap();
        assert_eq!(results.len(), 4); // 应该生成4个版本的激活码
        
        for activation_result in results {
            assert!(!activation_result.advanced_code.is_empty());
            assert!(!activation_result.professional_code.is_empty());
            assert_eq!(activation_result.advanced_code.len(), 16);
            assert_eq!(activation_result.professional_code.len(), 16);
        }
    }

    #[test]
    fn test_format_all_codes() {
        let machine_code = "ABC123DEF456";
        let result = ActivationCodeGenerator::format_all_codes(machine_code);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("FinalShell < 3.9.6"));
        assert!(formatted.contains("FinalShell ≥ 3.9.6"));
        assert!(formatted.contains("FinalShell 4.5"));
        assert!(formatted.contains("FinalShell 4.6"));
        assert!(formatted.contains("高级版"));
        assert!(formatted.contains("专业版"));
    }

    #[test]
    fn test_version_detection() {
        let short_code = "ABC123";
        let version = FinalShellVersion::detect_version(short_code);
        assert_eq!(version.version, "< 3.9.6");
        assert!(version.is_legacy);

        let long_code = "ABC123DEF456GHI789JKL012";
        let version = FinalShellVersion::detect_version(long_code);
        assert_ne!(version.version, "< 3.9.6");
        assert!(!version.is_legacy);
    }
}
