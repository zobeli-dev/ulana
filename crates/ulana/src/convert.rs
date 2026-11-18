//! Values between Luau and JSON, with the checks a script cannot get around.

use mlua::{Lua, Table, Value};
use serde_json::{Map, Number};

use crate::error::{Error, Result};

/// The most values one conversion walks, so a huge table cannot stall the host.
const WALK_LIMIT: usize = 200_000;
/// How deep tables may nest. Also stops a table that contains itself.
const DEPTH_LIMIT: usize = 64;

/// Turn a Luau value into JSON.
///
/// A table whose keys are exactly 1..n becomes an array, an empty table an
/// empty array, and a table with only string keys an object. Functions,
/// mixed tables, and numbers that are not finite are refused with the path.
pub(crate) fn to_json(value: &Value, path: &str) -> Result<serde_json::Value> {
    let mut seen = 0;
    walk(value, path, 0, &mut seen)
}

fn walk(value: &Value, path: &str, depth: usize, seen: &mut usize) -> Result<serde_json::Value> {
    *seen += 1;
    if *seen > WALK_LIMIT {
        return Err(Error::invalid(path, "too many values"));
    }
    Ok(match value {
        Value::Nil => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Integer(i) => serde_json::Value::Number((*i).into()),
        Value::Number(n) => number(*n)
            .ok_or_else(|| Error::invalid(path, "numbers must be finite (not NaN or infinity)"))?,
        Value::String(s) => match s.to_str() {
            Ok(s) => serde_json::Value::String(s.to_string()),
            Err(_) => return Err(Error::invalid(path, "text must be valid UTF-8")),
        },
        Value::Table(t) => {
            if depth >= DEPTH_LIMIT {
                return Err(Error::invalid(
                    path,
                    format!(
                        "tables may nest at most {DEPTH_LIMIT} deep (or a table contains itself)"
                    ),
                ));
            }
            table(t, path, depth, seen)?
        }
        Value::Function(_) => {
            return Err(Error::invalid(path, "a function cannot go here"));
        }
        other => {
            return Err(Error::invalid(
                path,
                format!("a {} cannot go here", other.type_name()),
            ))
        }
    })
}

fn table(t: &Table, path: &str, depth: usize, seen: &mut usize) -> Result<serde_json::Value> {
    let len = t.raw_len();
    let mut keys = 0usize;
    let mut named = false;
    for pair in t.pairs::<Value, Value>() {
        let (k, _) = pair.map_err(|e| Error::Runtime(e.to_string()))?;
        keys += 1;
        if matches!(k, Value::String(_)) {
            named = true;
        } else if !is_index(&k, len) {
            return Err(Error::invalid(
                path,
                "table keys must be text, or a list counting from 1",
            ));
        }
    }
    if named && keys > 0 && len > 0 {
        return Err(Error::invalid(
            path,
            "a table cannot mix list items and named fields",
        ));
    }
    if !named {
        if keys != len {
            return Err(Error::invalid(
                path,
                "a list cannot have gaps (nil in the middle)",
            ));
        }
        let mut items = Vec::with_capacity(len);
        for i in 1..=len {
            let v: Value = t.raw_get(i).map_err(|e| Error::Runtime(e.to_string()))?;
            items.push(walk(&v, &format!("{path}[{}]", i - 1), depth + 1, seen)?);
        }
        return Ok(serde_json::Value::Array(items));
    }
    let mut map = Map::new();
    for pair in t.pairs::<Value, Value>() {
        let (k, v) = pair.map_err(|e| Error::Runtime(e.to_string()))?;
        let Value::String(k) = k else {
            unreachable!("checked above")
        };
        let key = k
            .to_str()
            .map_err(|_| Error::invalid(path, "keys must be valid UTF-8"))?
            .to_string();
        let child = walk(&v, &field(path, &key), depth + 1, seen)?;
        map.insert(key, child);
    }
    Ok(serde_json::Value::Object(map))
}

fn is_index(k: &Value, len: usize) -> bool {
    match k {
        Value::Integer(i) => *i >= 1 && (*i as u64) <= len as u64,
        Value::Number(n) => n.fract() == 0.0 && *n >= 1.0 && *n <= len as f64,
        _ => false,
    }
}

fn number(n: f64) -> Option<serde_json::Value> {
    if !n.is_finite() {
        return None;
    }
    if n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 {
        return Some(serde_json::Value::Number(Number::from(n as i64)));
    }
    Number::from_f64(n).map(serde_json::Value::Number)
}

fn field(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

/// Turn JSON into a Luau value. `null` becomes `nil`.
pub(crate) fn from_json(lua: &Lua, value: &serde_json::Value) -> mlua::Result<Value> {
    Ok(match value {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value::String(lua.create_string(s)?),
        serde_json::Value::Array(items) => {
            let t = lua.create_table_with_capacity(items.len(), 0)?;
            for (i, item) in items.iter().enumerate() {
                t.raw_set(i + 1, from_json(lua, item)?)?;
            }
            Value::Table(t)
        }
        serde_json::Value::Object(map) => {
            let t = lua.create_table_with_capacity(0, map.len())?;
            for (k, v) in map {
                t.raw_set(k.as_str(), from_json(lua, v)?)?;
            }
            Value::Table(t)
        }
    })
}
