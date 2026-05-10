use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTemplate {
    pub name: String,
    pub module: String,
    pub description: String,
    pub signature: String,
    pub header_code: String,
    pub impl_code: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub starred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionGroup {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// true = built-in group (input/math/display/array), cannot be deleted
    #[serde(default)]
    pub builtin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryFile {
    pub module: String,
    pub functions: Vec<FunctionTemplate>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct GroupsFile {
    pub groups: Vec<FunctionGroup>,
}

#[derive(Debug, Default, Clone)]
pub struct FunctionLibrary {
    functions: Vec<FunctionTemplate>,
    pub groups: Vec<FunctionGroup>,
}

impl FunctionLibrary {
    pub fn load() -> Self {
        let mut lib = Self::default();

        // Built-in groups always present
        for name in ["input", "math", "display", "array"] {
            lib.groups.push(FunctionGroup {
                name: name.to_string(),
                description: String::new(),
                builtin: true,
            });
        }

        // Load built-in function TOML
        for (module, content) in BUILTIN_TOML_FILES {
            lib.load_toml_str(content, module);
        }

        // Load user groups from ~/.config/newc/groups.toml
        if let Some(path) = groups_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(file) = toml::from_str::<GroupsFile>(&content) {
                    for g in file.groups {
                        if !lib.groups.iter().any(|existing| existing.name == g.name) {
                            lib.groups.push(g);
                        }
                    }
                }
            }
        }

        // Load user function overrides from ~/.config/newc/functions/*.toml
        if let Some(user_dir) = user_functions_dir() {
            if let Ok(entries) = std::fs::read_dir(&user_dir) {
                let mut paths: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
                    .map(|e| e.path())
                    .collect();
                paths.sort();
                for path in paths {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let module = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        lib.load_toml_str(&content, &module);
                        // Auto-add group for any user module not already listed
                        if !lib.groups.iter().any(|g| g.name == module) {
                            lib.groups.push(FunctionGroup {
                                name: module,
                                description: String::new(),
                                builtin: false,
                            });
                        }
                    }
                }
            }
        }

        lib
    }

    fn load_toml_str(&mut self, content: &str, _module: &str) {
        let Ok(file): std::result::Result<LibraryFile, _> = toml::from_str(content) else {
            return;
        };
        for func in file.functions {
            self.functions.retain(|f| f.name != func.name);
            self.functions.push(func);
        }
    }

    // ── Groups ────────────────────────────────────────────────────────────────

    pub fn create_group(&mut self, name: impl Into<String>, description: impl Into<String>) -> bool {
        let name = name.into();
        if self.groups.iter().any(|g| g.name == name) {
            return false;
        }
        self.groups.push(FunctionGroup { name, description: description.into(), builtin: false });
        true
    }

    pub fn rename_group(&mut self, old: &str, new: impl Into<String>) -> bool {
        let new = new.into();
        if self.groups.iter().any(|g| g.name == new) {
            return false;
        }
        if let Some(g) = self.groups.iter_mut().find(|g| g.name == old && !g.builtin) {
            g.name = new.clone();
            // Rename module field on all affected functions
            for f in &mut self.functions {
                if f.module == old {
                    f.module = new.clone();
                }
            }
            return true;
        }
        false
    }

    /// Delete group. cascade=true also deletes all functions in the group.
    pub fn delete_group(&mut self, name: &str, cascade: bool) -> bool {
        let Some(pos) = self.groups.iter().position(|g| g.name == name && !g.builtin) else {
            return false;
        };
        self.groups.remove(pos);
        if cascade {
            self.functions.retain(|f| f.module != name);
        }
        true
    }

    pub fn save_groups(&self) -> Result<()> {
        let Some(path) = groups_path() else { return Ok(()) };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let user_groups: Vec<&FunctionGroup> = self.groups.iter().filter(|g| !g.builtin).collect();
        let file = GroupsFile {
            groups: user_groups.into_iter().cloned().collect(),
        };
        let content = toml::to_string_pretty(&file)
            .map_err(|e| crate::error::NewcError::Other(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn group_names(&self) -> Vec<String> {
        self.groups.iter().map(|g| g.name.clone()).collect()
    }

    // ── Functions ─────────────────────────────────────────────────────────────

    pub fn all(&self) -> &[FunctionTemplate] {
        &self.functions
    }

    pub fn by_module(&self, module: &str) -> Vec<&FunctionTemplate> {
        self.functions.iter().filter(|f| f.module == module).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&FunctionTemplate> {
        let q = query.to_lowercase();
        self.functions
            .iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&q)
                    || f.description.to_lowercase().contains(&q)
                    || f.tags.iter().any(|t| t.to_lowercase().contains(&q))
                    || f.module.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn modules(&self) -> Vec<String> {
        let mut mods: Vec<String> =
            self.functions.iter().map(|f| f.module.clone()).collect();
        mods.sort();
        mods.dedup();
        mods
    }

    pub fn upsert(&mut self, func: FunctionTemplate) {
        // Auto-register group if new
        if !self.groups.iter().any(|g| g.name == func.module) {
            self.groups.push(FunctionGroup {
                name: func.module.clone(),
                description: String::new(),
                builtin: false,
            });
        }
        self.functions.retain(|f| f.name != func.name);
        self.functions.push(func);
    }

    pub fn remove(&mut self, name: &str) {
        self.functions.retain(|f| f.name != name);
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut FunctionTemplate> {
        self.functions.iter_mut().find(|f| f.name == name)
    }

    pub fn resolve_deps(&self, selected: &[String]) -> Vec<String> {
        let mut resolved: Vec<String> = selected.to_vec();
        let mut i = 0;
        while i < resolved.len() {
            let name = resolved[i].clone();
            if let Some(func) = self.functions.iter().find(|f| f.name == name) {
                for req in &func.requires {
                    if !resolved.contains(req) {
                        resolved.push(req.clone());
                    }
                }
            }
            i += 1;
        }
        resolved
    }

    pub fn save_user_function(func: &FunctionTemplate) -> Result<()> {
        let Some(dir) = user_functions_dir() else { return Ok(()) };
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.toml", func.module));
        let mut file: LibraryFile = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            toml::from_str(&content).unwrap_or(LibraryFile {
                module: func.module.clone(),
                functions: Vec::new(),
            })
        } else {
            LibraryFile { module: func.module.clone(), functions: Vec::new() }
        };
        file.functions.retain(|f| f.name != func.name);
        file.functions.push(func.clone());
        let content = toml::to_string_pretty(&file)
            .map_err(|e| crate::error::NewcError::Other(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Scan C implementation code and return inferred `requires` entries.
/// Detects stdlib header needs and calls to other known library functions.
pub fn detect_requires(impl_code: &str, lib: &FunctionLibrary) -> Vec<String> {
    let mut reqs: Vec<String> = Vec::new();

    let stdlib: &[(&str, &str)] = &[
        // stdio
        ("printf(",   "stdio.h"), ("fprintf(",  "stdio.h"), ("scanf(",    "stdio.h"),
        ("fscanf(",   "stdio.h"), ("sscanf(",   "stdio.h"), ("fgets(",    "stdio.h"),
        ("fopen(",    "stdio.h"), ("fclose(",   "stdio.h"), ("fread(",    "stdio.h"),
        ("fwrite(",   "stdio.h"), ("puts(",     "stdio.h"), ("getchar(",  "stdio.h"),
        ("putchar(",  "stdio.h"), ("snprintf(", "stdio.h"), ("sprintf(",  "stdio.h"),
        ("perror(",   "stdio.h"), ("rewind(",   "stdio.h"), ("feof(",     "stdio.h"),
        ("FILE",      "stdio.h"), ("EOF",       "stdio.h"),
        // stdlib
        ("malloc(",   "stdlib.h"), ("calloc(",  "stdlib.h"), ("realloc(", "stdlib.h"),
        ("free(",     "stdlib.h"), ("exit(",    "stdlib.h"), ("abort(",   "stdlib.h"),
        ("atoi(",     "stdlib.h"), ("atof(",    "stdlib.h"), ("atol(",    "stdlib.h"),
        ("rand(",     "stdlib.h"), ("srand(",   "stdlib.h"), ("abs(",     "stdlib.h"),
        ("qsort(",    "stdlib.h"), ("bsearch(", "stdlib.h"), ("getenv(",  "stdlib.h"),
        // string
        ("strlen(",   "string.h"), ("strcpy(",  "string.h"), ("strncpy(", "string.h"),
        ("strcmp(",   "string.h"), ("strncmp(", "string.h"), ("strcat(",  "string.h"),
        ("strncat(",  "string.h"), ("strchr(",  "string.h"), ("strrchr(", "string.h"),
        ("strstr(",   "string.h"), ("memset(",  "string.h"), ("memcpy(",  "string.h"),
        ("memcmp(",   "string.h"), ("memmove(", "string.h"),
        // math
        ("sqrt(",  "math.h"), ("pow(",   "math.h"), ("fabs(",  "math.h"),
        ("sin(",   "math.h"), ("cos(",   "math.h"), ("tan(",   "math.h"),
        ("asin(",  "math.h"), ("acos(",  "math.h"), ("atan(",  "math.h"),
        ("atan2(", "math.h"), ("log(",   "math.h"), ("log2(",  "math.h"),
        ("log10(", "math.h"), ("exp(",   "math.h"), ("floor(", "math.h"),
        ("ceil(",  "math.h"), ("round(", "math.h"), ("fmod(",  "math.h"),
        // ctype
        ("isdigit(", "ctype.h"), ("isalpha(", "ctype.h"), ("isspace(",  "ctype.h"),
        ("toupper(", "ctype.h"), ("tolower(", "ctype.h"), ("isalnum(",  "ctype.h"),
        ("ispunct(", "ctype.h"), ("isupper(", "ctype.h"), ("islower(",  "ctype.h"),
        // time
        ("time(",      "time.h"), ("clock(",     "time.h"), ("difftime(", "time.h"),
        ("localtime(", "time.h"), ("strftime(",  "time.h"), ("mktime(",   "time.h"),
        // assert
        ("assert(", "assert.h"),
        // stdarg
        ("va_list", "stdarg.h"), ("va_start(", "stdarg.h"), ("va_end(", "stdarg.h"),
        // stdbool
        ("bool ", "stdbool.h"), ("true", "stdbool.h"), ("false", "stdbool.h"),
    ];

    for (pattern, header) in stdlib {
        if impl_code.contains(pattern) {
            let h = header.to_string();
            if !reqs.contains(&h) {
                reqs.push(h);
            }
        }
    }

    // Library function calls
    for func in lib.all() {
        let call = format!("{}(", func.name);
        if impl_code.contains(&call) {
            let n = func.name.clone();
            if !reqs.contains(&n) {
                reqs.push(n);
            }
        }
    }

    reqs
}

fn user_functions_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("newc").join("functions"))
}

fn groups_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("newc").join("groups.toml"))
}

const BUILTIN_TOML_FILES: &[(&str, &str)] = &[
    ("input",       include_str!("../../assets/functions/input.toml")),
    ("math",        include_str!("../../assets/functions/math.toml")),
    ("display",     include_str!("../../assets/functions/display.toml")),
    ("array",       include_str!("../../assets/functions/array.toml")),
    ("algorithms",  include_str!("../../assets/functions/algorithms.toml")),
    ("strings",     include_str!("../../assets/functions/strings.toml")),
    ("linked_list", include_str!("../../assets/functions/linked_list.toml")),
    ("files",       include_str!("../../assets/functions/files.toml")),
    ("test_utils",  include_str!("../../assets/functions/test_utils.toml")),
];
