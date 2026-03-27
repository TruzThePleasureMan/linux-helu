with open("helud/src/auth/mod.rs", "r") as f:
    content = f.read()

content = content.replace(
"""            if let Some(m) = self.methods.get("face") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("fingerprint") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("pin") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }""",
"""            if let Some(m) = self.methods.get("face") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("fingerprint") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("pin") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }"""
)

old = """            if let Some(m) = self.methods.get("face") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("fingerprint") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("pin") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }"""

new = """            if let Some(m) = self.methods.get("face") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("fingerprint") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("pin") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }"""

import sys

with open("helud/src/auth/mod.rs", "r") as f:
    content = f.read()

old3 = "            if let Some(m) = self.methods.get(\"face\") {\n                if m.authenticate(username).unwrap_or(false) { return Ok(true); }\n            }\n            if let Some(m) = self.methods.get(\"fingerprint\") {\n                if m.authenticate(username).unwrap_or(false) { return Ok(true); }\n            }\n            if let Some(m) = self.methods.get(\"pin\") {\n                if m.authenticate(username).unwrap_or(false) { return Ok(true); }\n            }"
new3 = "            if let Some(m) = self.methods.get(\"face\") {\n                if m.authenticate(username).unwrap_or(false) { return Ok(true); }\n            }\n            if let Some(m) = self.methods.get(\"fingerprint\") {\n                if m.authenticate(username).unwrap_or(false) { return Ok(true); }\n            }\n            if let Some(m) = self.methods.get(\"pin\") {\n                if m.authenticate(username).unwrap_or(false) { return Ok(true); }\n            }"

new4 = """            if let Some(m) = self.methods.get("face") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("fingerprint") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }
            if let Some(m) = self.methods.get("pin") {
                if m.authenticate(username).unwrap_or(false) { return Ok(true); }
            }"""

new5 = """            if self.methods.get("face").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            if self.methods.get("fingerprint").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }
            if self.methods.get("pin").is_some_and(|m| m.authenticate(username).unwrap_or(false)) { return Ok(true); }"""

content = content.replace(old3, new5)

with open("helud/src/auth/mod.rs", "w") as f:
    f.write(content)
