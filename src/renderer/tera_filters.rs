use anyhow::Result;
use serde::{Deserialize, Serialize};
use tera::{Value, from_value, to_value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HSLColor {
    /// 0..=359
    pub hue: u16,
    /// 0..=100
    pub saturation: u8,
    /// 0..=100
    pub lightness: u8,
}

impl Serialize for HSLColor {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(
            "hsl({} {}% {}%)",
            self.hue, self.saturation, self.lightness
        ))
    }
}

struct HSLVisitor;

impl<'de> serde::de::Visitor<'de> for HSLVisitor {
    type Value = HSLColor;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(r#"a string like "hsl(210 60% 50%)""#)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // Accept "hsl(H S% L%)" or "hsl(H, S%, L%)" (commas optional)
        let s = v.trim();
        if !s.starts_with("hsl(") || !s.ends_with(')') {
            return Err(E::custom("expected hsl(...)"));
        }
        let inner = &s[4..s.len() - 1];
        // Replace commas with spaces, collapse whitespace.
        let cleaned = inner.replace(',', " ");
        let parts: Vec<&str> = cleaned.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(E::custom("expected 3 values: H S% L%"));
        }

        let hue: u16 = parts[0].parse().map_err(|_| E::custom("bad hue"))?;
        let sat = parts[1]
            .strip_suffix('%')
            .ok_or_else(|| E::custom("S must end with %"))?;
        let lig = parts[2]
            .strip_suffix('%')
            .ok_or_else(|| E::custom("L must end with %"))?;
        let saturation: u8 = sat.parse().map_err(|_| E::custom("bad saturation"))?;
        let lightness: u8 = lig.parse().map_err(|_| E::custom("bad lightness"))?;

        Ok(HSLColor::new(hue, saturation, lightness))
    }
}

impl<'de> Deserialize<'de> for HSLColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(HSLVisitor)
    }
}

impl HSLColor {
    pub fn new(hue: u16, saturation: u8, lightness: u8) -> Self {
        Self {
            hue: hue % 360,
            saturation: saturation.min(100),
            lightness: lightness.min(100),
        }
    }
    pub fn invert(&self) -> Self {
        let inv_h = (self.hue + 180) % 360;
        let inv_l = 100u16.saturating_sub(self.lightness as u16) as u8;
        Self::new(inv_h, self.saturation, inv_l)
    }

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        hsl_to_rgb(self.hue, self.saturation, self.lightness)
    }

    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb();
        format!("#{r:02X}{g:02X}{b:02X}")
    }
}

fn hsl_to_rgb(h: u16, s_pct: u8, l_pct: u8) -> (u8, u8, u8) {
    let h = h as f32;
    let s = s_pct as f32 / 100.0;
    let l = l_pct as f32 / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h {
        h if (0.0..60.0).contains(&h) => (c, x, 0.0),
        h if (60.0..120.0).contains(&h) => (x, c, 0.0),
        h if (120.0..180.0).contains(&h) => (0.0, c, x),
        h if (180.0..240.0).contains(&h) => (0.0, x, c),
        h if (240.0..300.0).contains(&h) => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let (r, g, b) = (r1 + m, g1 + m, b1 + m);
    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn rel_luminance((r, g, b): (u8, u8, u8)) -> f32 {
    fn ch(c: u8) -> f32 {
        let c = c as f32 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    let (r, g, b) = (ch(r), ch(g), ch(b));
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast_ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f32 {
    let (mut l1, mut l2) = (rel_luminance(a), rel_luminance(b));
    if l1 < l2 {
        std::mem::swap(&mut l1, &mut l2)
    }
    (l1 + 0.05) / (l2 + 0.05)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn word_to_hsl_color(word: &str) -> HSLColor {
    let h = fnv1a64(word.as_bytes());
    let hue = (h as u16) % 360;

    let sat_raw = ((h >> 16) & 0xFF) as u8;
    let lig_raw = ((h >> 24) & 0xFF) as u8;

    let saturation = 55 + (sat_raw % 31); // 55..85
    let lightness = 40 + (lig_raw % 21); // 40..60

    HSLColor::new(hue, saturation, lightness)
}

pub fn text_color_for_bg(bg: HSLColor) -> &'static str {
    let rgb = bg.to_rgb();
    let c_white = contrast_ratio(rgb, (255, 255, 255));
    let c_black = contrast_ratio(rgb, (0, 0, 0));
    if c_white >= c_black { "white" } else { "black" }
}

/// {{ name | word_to_color }} -> "#RRGGBB"
pub fn word_to_color_filter(
    value: &Value,
    _args: &std::collections::HashMap<String, Value>,
) -> tera::Result<Value> {
    let s: String = from_value(value.clone()).map_err(|e| tera::Error::msg(e.to_string()))?;
    let hsl = word_to_hsl_color(&s);
    Ok(to_value(hsl.to_hex()).unwrap())
}

/// {{ name | word_text_color }} -> "black" or "white"
pub fn word_text_color_filter(
    value: &Value,
    _args: &std::collections::HashMap<String, Value>,
) -> tera::Result<Value> {
    let s: String = from_value(value.clone()).map_err(|e| tera::Error::msg(e.to_string()))?;
    let hsl = word_to_hsl_color(&s);
    Ok(to_value(text_color_for_bg(hsl)).unwrap())
}

/// {{ name | word_hsl }} -> "hsl(H S% L%)"
pub fn word_hsl_filter(
    value: &Value,
    _args: &std::collections::HashMap<String, Value>,
) -> tera::Result<Value> {
    let s: String = from_value(value.clone()).map_err(|e| tera::Error::msg(e.to_string()))?;
    let hsl = word_to_hsl_color(&s);
    Ok(to_value(
        serde_json::to_string(&hsl)
            .unwrap_or_else(|_| format!("hsl({} {}% {}%)", hsl.hue, hsl.saturation, hsl.lightness)),
    )
    .unwrap())
}

/// {{ [1, 2, 3] | pop }} -> [1, 2]
/// {{ ["a", "b", "c"] | pop(value="b") }} -> ["a", "c"]
pub fn pop_filter(
    value: &Value,
    args: &std::collections::HashMap<String, Value>,
) -> tera::Result<Value> {
    let mut arr = match value {
        Value::Array(a) => a.clone(),
        _ => return Err(tera::Error::msg("Filter argument must be an array")),
    };

    if let Some(target) = args.get("value") {
        arr.retain(|x| x != target);
    } else {
        arr.pop();
    }

    Ok(Value::Array(arr))
}
