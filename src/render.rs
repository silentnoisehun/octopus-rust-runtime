use crate::orchestration::{ArmRecord, ArmStatus, RootRecord};
use std::io::IsTerminal;

pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}

pub fn no_color() -> bool {
    std::env::var_os("NO_COLOR")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

pub fn visual_active(plain_flag: bool) -> bool {
    !plain_flag && is_tty() && !no_color()
}

pub fn state_glyph(status: &ArmStatus) -> &'static str {
    match status {
        ArmStatus::Running => "\u{25c7}",
        ArmStatus::Completed => "\u{2713}",
        ArmStatus::Failed => "\u{26a0}",
        ArmStatus::Cancelled => "\u{26a0}",
        ArmStatus::TimedOut => "\u{26a0}",
        ArmStatus::Resumed => "\u{25c7}",
    }
}

pub fn heartbeat_glyph(status: &ArmStatus) -> &'static str {
    match status {
        ArmStatus::Completed => "\u{25cf}",
        ArmStatus::Running => "\u{25c7}",
        _ => "\u{25cb}",
    }
}

pub fn color_marker(status: &ArmStatus) -> &'static str {
    match status {
        ArmStatus::Completed => "\u{1f7e9}",
        ArmStatus::Running => "\u{1f7e8}",
        ArmStatus::Failed => "\u{1f7e5}",
        ArmStatus::Cancelled => "\u{1f7e5}",
        ArmStatus::TimedOut => "\u{1f7e5}",
        ArmStatus::Resumed => "\u{1f7e8}",
    }
}

pub const MERKLE_MARKER: &str = "\u{1f7ea}";
pub const CRYO_MARKER: &str = "\u{1f7e6}";

pub fn density_bar(label: &str, fill: u8) -> String {
    let fill = fill.min(8);
    let blocks: String = (0..fill).map(|_| '\u{2593}').collect();
    let empty: String = (fill..8).map(|_| '\u{2591}').collect();
    format!("{blocks}{empty}({label})")
}

pub fn arm_density_bar(arm: &ArmRecord) -> String {
    let fill = match arm.status {
        ArmStatus::Completed => 8,
        ArmStatus::Running => 3,
        ArmStatus::Resumed => 5,
        ArmStatus::Failed | ArmStatus::Cancelled | ArmStatus::TimedOut => 1,
    };
    density_bar(&arm.name, fill)
}

pub fn root_density_bar(root: &RootRecord) -> String {
    let fill = match root.status {
        ArmStatus::Completed => 8,
        ArmStatus::Running => 3,
        _ => 1,
    };
    density_bar("O", fill)
}

pub fn binary_capacity_bar(filled: usize, total: usize) -> String {
    let total = total.min(10);
    let filled = filled.min(total);
    let blocks: String = (0..filled).map(|_| '\u{25a0}').collect();
    let empty: String = (filled..total).map(|_| '\u{25a1}').collect();
    format!("{blocks}{empty}")
}

pub fn render_status_octopus(root: &RootRecord, arms: &[ArmRecord]) -> String {
    let mut out = String::new();

    let dur = root
        .duration_ms
        .map(|d| format!("{d}ms"))
        .unwrap_or_else(|| "-".to_string());
    out.push_str(&format!(
        "{} {} {} {}  {}  {}\n",
        MERKLE_MARKER,
        root_density_bar(root),
        heartbeat_glyph(&root.status),
        root.id,
        root.status.as_str(),
        dur,
    ));

    let failed = arms
        .iter()
        .filter(|a| a.status == ArmStatus::Failed)
        .count();
    let total_arms = arms.len().max(1);
    out.push_str(&format!(
        "{} {} {} failed\n",
        if failed > 0 { "\u{1f7e5}" } else { "\u{1f7e9}" },
        binary_capacity_bar(failed, total_arms),
        failed,
    ));

    for (i, arm) in arms.iter().enumerate() {
        let is_last = i == arms.len() - 1;
        let connector = "           ";
        let dur = arm
            .duration_ms
            .map(|d| format!("{d}ms"))
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "{}\u{25ef} {} {} {} {} {} {} \u{2500}\u{2500}\u{2524}\n",
            connector,
            color_marker(&arm.status),
            arm_density_bar(arm),
            &arm.name,
            state_glyph(&arm.status),
            dur,
            arm.status.as_str(),
        ));

        let children: Vec<&ArmRecord> = arms
            .iter()
            .filter(|a| a.parent_arm_id.as_deref() == Some(&arm.id))
            .collect();
        for (j, child) in children.iter().enumerate() {
            let sub_connector = if is_last {
                "                "
            } else {
                "               "
            };
            let child_prefix = if j == children.len() - 1 {
                "\u{2514}\u{2500}\u{2500} "
            } else {
                "\u{251c}\u{2500}\u{2500} "
            };
            let dur = child
                .duration_ms
                .map(|d| format!("{d}ms"))
                .unwrap_or_else(|| "-".to_string());
            out.push_str(&format!(
                "{}{}\u{25ef} {} {} {} {} {} {} \u{2500}\u{2500}\u{2524}\n",
                sub_connector,
                child_prefix,
                color_marker(&child.status),
                arm_density_bar(child),
                &child.name,
                state_glyph(&child.status),
                dur,
                child.status.as_str(),
            ));
        }
    }

    out
}

pub fn render_pipeline_octopus(root_id: &str, arms: &[ArmRecord]) -> String {
    let completed = arms.iter().all(|a| a.status == ArmStatus::Completed);
    let any_failed = arms.iter().any(|a| a.status == ArmStatus::Failed);
    let status = if any_failed {
        ArmStatus::Failed
    } else if completed {
        ArmStatus::Completed
    } else {
        ArmStatus::Running
    };
    let dur = arms.iter().filter_map(|a| a.duration_ms).max();
    let root = RootRecord {
        id: root_id.to_string(),
        status,
        prompt_hash: String::new(),
        input_hash: String::new(),
        output_hash: None,
        started_at: 0,
        finished_at: None,
        duration_ms: dur,
        children: arms.iter().map(|a| a.id.clone()).collect(),
    };
    render_status_octopus(&root, arms)
}

pub fn render_arm_octopus(root_id: &str, arm: &ArmRecord) -> String {
    render_pipeline_octopus(root_id, std::slice::from_ref(arm))
}

pub fn ansi_clear_lines(n: usize) -> String {
    format!("\x1b[{n}A\x1b[J")
}

pub fn ansi_bright_on() -> &'static str {
    "\x1b[1m"
}

pub fn ansi_reset() -> &'static str {
    "\x1b[0m"
}
