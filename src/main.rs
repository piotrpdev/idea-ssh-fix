use anyhow::Context;
use inotify::{Inotify, WatchMask};
use std::{fs, path::Path, sync::LazyLock};

const APPLICATIONS_DIR: &str = "/home/pplaczek/.local/share/applications";
const IDEA_DESKTOP_FILE_NAME_PREFIX: &str = "jetbrains-idea";
const IDEA_DESKTOP_FILE_NAME_SUFFIX: &str = ".desktop";
const REPLACE_PATTERN: &str = "Exec=";
const REPLACEMENT: &str = "Exec=idea %u";
const INOTIFY_EVENT_BUFFER_SIZE: usize = 4096;
static INOTIFY_EVENT_MASK: LazyLock<WatchMask> =
    LazyLock::new(|| WatchMask::CLOSE_WRITE | WatchMask::MOVED_TO);

fn replace_exec(idea_desktop_file_path: &std::path::Path) -> anyhow::Result<()> {
    let content = fs::read_to_string(idea_desktop_file_path)?;

    let mut needs_replacement = false;

    let new_content = content
        .lines()
        .map(|line| {
            if !line.contains(REPLACE_PATTERN) || line == REPLACEMENT {
                return line;
            }

            needs_replacement = true;
            REPLACEMENT
        })
        .collect::<Vec<_>>()
        .join("\n");

    if !needs_replacement {
        println!("Exec line does not need replacement, skipping.");
        return Ok(());
    }

    fs::write(idea_desktop_file_path, new_content)
        .context("Failed to write modified content to desktop file")?;

    println!("Exec line replaced successfully.");

    Ok(())
}

#[allow(clippy::unnecessary_debug_formatting)]
fn main() -> anyhow::Result<()> {
    let applications_dir_path = Path::new(APPLICATIONS_DIR);
    let mut inotify = Inotify::init().context("Failed to initialize inotify")?;

    let idea_desktop_file_path = fs::read_dir(APPLICATIONS_DIR)?
        .find(|entry| {
            entry.as_ref().is_ok_and(|f| {
                let name_string = f.file_name().to_string_lossy().into_owned();
                name_string.starts_with(IDEA_DESKTOP_FILE_NAME_PREFIX)
                    && name_string.ends_with(IDEA_DESKTOP_FILE_NAME_SUFFIX)
            })
        })
        .context("Couldn't find JetBrains IDEA desktop file")??
        .path();

    println!("Found JetBrains IDEA desktop file at: {idea_desktop_file_path:?}");
    println!("Attempting to replace Exec line in desktop file... ");
    replace_exec(&idea_desktop_file_path).context("Failed to replace Exec line in desktop file")?;

    inotify
        .watches()
        .add(APPLICATIONS_DIR, *INOTIFY_EVENT_MASK)
        .context("Failed to add inotify watch")?;

    println!("Watching {APPLICATIONS_DIR} for changes...");

    let mut buffer = [0u8; INOTIFY_EVENT_BUFFER_SIZE];
    loop {
        let all_events = inotify
            .read_events_blocking(&mut buffer)
            .context("Failed to read inotify events")?;

        let idea_events = all_events.filter(|event| {
            event.name.as_ref().is_some_and(|name| {
                let name_string = name.to_string_lossy();
                name_string.starts_with(IDEA_DESKTOP_FILE_NAME_PREFIX)
                    && name_string.ends_with(IDEA_DESKTOP_FILE_NAME_SUFFIX)
            })
        });

        for event in idea_events {
            println!("Event: {event:?}");

            let Some(file_name) = event.name else {
                println!("Event has no file name, skipping.");
                continue;
            };

            println!("Attempting to replace Exec line in desktop file... ");
            replace_exec(&applications_dir_path.join(file_name))
                .context("Failed to replace Exec line in desktop file after modification")?;
        }
    }
}
