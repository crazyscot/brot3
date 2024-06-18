// Tauri-facing side of brot3
// (c) 2024 Ross Younger

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod jobs;
mod menu;
mod mutable_util;
mod render_spec;
mod save_image;
mod tile_bridge;
mod util;
mod viewertilespec;

use jobs::OutstandingJobs;
use save_image::SaveState;
use viewertilespec::ViewerTileSpec;

use brot3_cli::styles;
use clap::{Parser, Subcommand};

#[derive(Clone, Debug, Subcommand)]
enum GuiCommands {
    #[command()]
    /// Help on extended CLI options
    HelpCli,
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Yet Another fractal plotter. Run without arguments for an interactive browser."
)]
//#[command(disable_help_flag = true)]
#[command(styles=styles::get())]
struct GuiCli {
    #[command(subcommand)]
    command: Option<GuiCommands>,
}

fn main() -> anyhow::Result<()> {
    #![allow(clippy::disallowed_types)]

    let ego = std::env::args_os().next().unwrap_or("brot3".into());

    let result = GuiCli::try_parse().map(|gui_cli| {
        if let Some(cmd) = gui_cli.command {
            match cmd {
                GuiCommands::HelpCli => {
                    // Send --help !
                    brot3_cli::Cli::parse_from([ego, "help".into()]);
                }
            }
            Ok(())
        } else {
            Err("no subcommand") // we should run the GUI
        }
    });
    // result is now regrettably twisted ...
    match result {
        // Outer Err comes from our parser
        Err(e) => match e.kind() {
            // Handle outer Version and Help
            clap::error::ErrorKind::DisplayVersion | clap::error::ErrorKind::DisplayHelp => {
                return e
                    .print()
                    .map(|_| ())
                    .or_else(|e| {
                        println!("Error printing parser output: {e:?}");
                        //Result::<(), Error>::Ok(())
                        Ok::<(), std::io::Error>(())
                    })
                    .map_err(|e: std::io::Error| anyhow::Error::new(e));
            }
            // Anything else we didn't recognise should be passed on through to the CLI
            _ => {}
        },
        // Ok(Ok) means we handled it
        Ok(Ok(_)) => {
            return Ok(()); // We're done here
        }
        // Ok(Err) means there was no subcommand, so drop on through and run the GUI
        Ok(Err(_)) => {}
    }

    // If we reach here: either there were no args (so run the GUI), or there were (run the CLI).
    if std::env::args_os().count() > 1 {
        return brot3_cli::cli_main();
    }

    let my_menu = menu::ApplicationMenu::new();
    tauri::Builder::default()
        .manage(OutstandingJobs::default())
        .manage(SaveState::default())
        .invoke_handler(tauri::generate_handler![
            save_image::save_image_workflow,
            tile_bridge::start_tile,
            tile_bridge::abort_tile,
            tile_bridge::get_metadata,
            tile_bridge::list_items,
        ])
        .menu(my_menu.build())
        .on_menu_event(move |event| my_menu.on_menu(event))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
