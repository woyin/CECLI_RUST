//! # 技能系统
//!
//! 技能包为代理提供领域知识、上下文规则和工具绑定。
//! 支持从目录中自动发现 `SKILL.md` 格式的技能定义。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 技能包定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillPack {
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 版本号
    pub version: Option<String>,
    /// 规则列表
    pub rules: Vec<String>,
    /// 上下文文件路径
    pub context_files: Vec<PathBuf>,
    /// 关联工具列表
    pub tools: Vec<String>,
    /// 技能来源
    pub source: SkillSource,
    /// 是否启用
    pub enabled: bool,
}

/// 技能来源
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillSource {
    /// 内置技能
    Builtin,
    /// 用户全局技能
    UserGlobal,
    /// 项目级技能
    Project,
}

/// 技能注册表
pub struct SkillRegistry {
    skills: Vec<SkillPack>,
}

impl SkillRegistry {
    /// 创建空的技能注册表
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    /// 注册技能包（同名则覆盖）
    pub fn register(&mut self, skill: SkillPack) {
        self.skills.retain(|s| s.name != skill.name);
        self.skills.push(skill);
    }

    /// 按名称获取技能包
    pub fn get(&self, name: &str) -> Option<&SkillPack> {
        self.skills.iter().find(|s| s.name == name)
    }

    /// 列出所有已启用的技能包
    pub fn list_enabled(&self) -> Vec<&SkillPack> {
        self.skills.iter().filter(|s| s.enabled).collect()
    }

    /// 启用指定技能，返回是否找到
    pub fn enable(&mut self, name: &str) -> bool {
        if let Some(s) = self.skills.iter_mut().find(|s| s.name == name) {
            s.enabled = true;
            true
        } else {
            false
        }
    }

    /// 禁用指定技能，返回是否找到
    pub fn disable(&mut self, name: &str) -> bool {
        if let Some(s) = self.skills.iter_mut().find(|s| s.name == name) {
            s.enabled = false;
            true
        } else {
            false
        }
    }

    /// 从目录发现技能包
    ///
    /// 目录结构: `<dir>/<name>/SKILL.md`
    ///
    /// `SKILL.md` 使用 YAML frontmatter 定义元数据，正文中以 `- ` 开头的行作为规则。
    pub fn discover_from_dir(
        &mut self,
        dir: &Path,
        source: SkillSource,
    ) -> Result<usize, std::io::Error> {
        let mut count = 0;
        if !dir.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    let content = std::fs::read_to_string(&skill_file)?;
                    if let Some(mut skill) = parse_skill_md(&content) {
                        skill.source = source.clone();
                        skill.context_files = vec![skill_file];
                        self.register(skill);
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// 收集所有已启用技能的规则
    pub fn collect_rules(&self) -> Vec<String> {
        self.skills
            .iter()
            .filter(|s| s.enabled)
            .flat_map(|s| s.rules.clone())
            .collect()
    }

    /// 返回已注册技能数量
    pub fn count(&self) -> usize {
        self.skills.len()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析 SKILL.md 文件内容
///
/// 格式示例：
/// ```text
/// ---
/// name: rust-expert
/// description: Rust programming expertise
/// version: 1.0
/// tools: [cargo, rustfmt]
/// ---
///
/// # Rules
///
/// - Always use Result for error handling
/// ```
fn parse_skill_md(content: &str) -> Option<SkillPack> {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        return None;
    }

    // 分割 frontmatter 和正文
    let after_first = &trimmed[3..];
    let end_idx = after_first.find("---")?;
    let frontmatter = after_first[..end_idx].trim();
    let body = after_first[end_idx + 3..].trim();

    // 解析 frontmatter 字段
    let mut name = String::new();
    let mut description = String::new();
    let mut version = None;
    let mut tools = Vec::new();

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("name:") {
            name = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("description:") {
            description = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("version:") {
            version = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("tools:") {
            let val = val.trim();
            // 解析 [tool1, tool2] 格式
            if val.starts_with('[') && val.ends_with(']') {
                let inner = &val[1..val.len() - 1];
                tools = inner
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            }
        }
    }

    if name.is_empty() {
        return None;
    }

    // 从正文提取规则（以 `- ` 开头的行）
    let rules: Vec<String> = body
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- ").map(|r| r.to_string()))
        .collect();

    Some(SkillPack {
        name,
        description,
        version,
        rules,
        context_files: Vec::new(),
        tools,
        source: SkillSource::Builtin,
        enabled: true,
    })
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 辅助函数：构造技能包
    fn make_skill(name: &str, enabled: bool) -> SkillPack {
        SkillPack {
            name: name.to_string(),
            description: format!("{name} description"),
            version: Some("1.0".to_string()),
            rules: vec![format!("rule from {name}")],
            context_files: Vec::new(),
            tools: Vec::new(),
            source: SkillSource::Builtin,
            enabled,
        }
    }

    #[test]
    fn test_register_skill() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("s1", true));
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn test_get_skill() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("alpha", true));
        assert!(reg.get("alpha").is_some());
        assert!(reg.get("beta").is_none());
    }

    #[test]
    fn test_list_enabled() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("on", true));
        reg.register(make_skill("off", false));

        let enabled = reg.list_enabled();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "on");
    }

    #[test]
    fn test_enable_disable() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("s1", true));

        assert!(reg.disable("s1"));
        assert!(reg.list_enabled().is_empty());

        assert!(reg.enable("s1"));
        assert_eq!(reg.list_enabled().len(), 1);

        // 操作不存在的技能应返回 false
        assert!(!reg.enable("nonexistent"));
        assert!(!reg.disable("nonexistent"));
    }

    #[test]
    fn test_collect_rules() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("s1", true));
        reg.register(make_skill("s2", false));
        reg.register(make_skill("s3", true));

        let rules = reg.collect_rules();
        assert_eq!(rules.len(), 2);
        assert!(rules.contains(&"rule from s1".to_string()));
        assert!(rules.contains(&"rule from s3".to_string()));
    }

    #[test]
    fn test_discover_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("rust-expert");
        fs::create_dir_all(&skill_dir).unwrap();

        let skill_md = "\
---
name: rust-expert
description: Rust programming expertise
version: 1.0
tools: [cargo, rustfmt]
---

# Rust Expert Rules

- Always use Result for error handling
- Prefer &str over String for function parameters
";
        fs::write(skill_dir.join("SKILL.md"), skill_md).unwrap();

        let mut reg = SkillRegistry::new();
        let count = reg
            .discover_from_dir(dir.path(), SkillSource::Project)
            .unwrap();
        assert_eq!(count, 1);

        let skill = reg.get("rust-expert").unwrap();
        assert_eq!(skill.description, "Rust programming expertise");
        assert_eq!(skill.version, Some("1.0".to_string()));
        assert_eq!(skill.tools, vec!["cargo", "rustfmt"]);
        assert_eq!(skill.rules.len(), 2);
        assert_eq!(skill.source, SkillSource::Project);
    }

    #[test]
    fn test_skill_md_parsing() {
        let content = "\
---
name: test-skill
description: A test skill
version: 2.0
tools: [tool1]
---

# Rules

- Rule one
- Rule two
- Rule three
";
        let skill = parse_skill_md(content).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.version, Some("2.0".to_string()));
        assert_eq!(skill.tools, vec!["tool1"]);
        assert_eq!(skill.rules.len(), 3);
    }

    #[test]
    fn test_duplicate_name_override() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("dup", true));
        reg.register(SkillPack {
            name: "dup".to_string(),
            description: "new description".to_string(),
            version: Some("2.0".to_string()),
            rules: vec![],
            context_files: Vec::new(),
            tools: Vec::new(),
            source: SkillSource::Project,
            enabled: true,
        });

        assert_eq!(reg.count(), 1);
        assert_eq!(reg.get("dup").unwrap().description, "new description");
    }

    #[test]
    fn test_count() {
        let mut reg = SkillRegistry::new();
        assert_eq!(reg.count(), 0);
        reg.register(make_skill("a", true));
        assert_eq!(reg.count(), 1);
        reg.register(make_skill("b", true));
        assert_eq!(reg.count(), 2);
    }

    #[test]
    fn test_empty_registry() {
        let reg = SkillRegistry::new();
        assert_eq!(reg.count(), 0);
        assert!(reg.list_enabled().is_empty());
        assert!(reg.get("x").is_none());
        assert!(reg.collect_rules().is_empty());
    }
}
