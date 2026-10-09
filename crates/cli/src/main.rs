// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 00faithlss

use anyhow::Result;
use icaro_core::manifest;

const TARGET_VERSION: &str = "1.21.1";

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("icaro/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let manifest = manifest::fetch_manifest(&client).await?;
    let entry = manifest.find(TARGET_VERSION)?;
    let json = manifest::fetch_version_json(&client, entry).await?;

    println!(
        "Minecraft {} ({}): JSON de {} bytes, SHA-1 verificado",
        entry.id,
        entry.kind,
        json.len()
    );
    Ok(())
}
