use core::time;
use std::{
    env,
    env::var,
    env::set_var,
    process::exit,
    thread,
};

use hass_rs::client;
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_output::WlOutput, wl_registry},
    Connection, QueueHandle,
};
use colors_transform::{self, Color, Hsl};
use serde_json::json;
use dotenv::dotenv;

mod clap;
mod output;
mod prominent_color;
mod backend;

struct AppState;

impl wayland_client::Dispatch<wl_registry::WlRegistry, GlobalListContents> for AppState {
    fn event(
        _: &mut AppState,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<AppState>,
    ) {
    }
}



#[cfg_attr(feature = "async-std-runtime", async_std::main)]
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match dotenv() {
        // Only warn on failure
        Err(err) => println!("Warning: Could not load .env {:?}", err),
        _ => {},
    }
    let args = clap::set_flags().get_matches();

    if args.is_present("debug") {
        set_var("RUST_LOG", "pxlha=trace");
    } else if env::var("RUST_LOG").is_err() {
        set_var("RUST_LOG", "pxlha=info");
    }

    let pause_duration = var("PAUSE_DURATION").unwrap_or("100".to_string()).parse::<u64>().expect("Could not parse PAUSE_DURATION");

    env_logger::init();
    log::trace!("Logger initialized.");

    // Display setup
    let mut conn = Connection::connect_to_env().unwrap();
    let (mut globals, _) = registry_queue_init::<AppState>(&conn).unwrap();

    if args.is_present("listoutputs") {
        let valid_outputs = output::get_all_outputs(&mut globals, &mut conn);
        for output in valid_outputs {
            log::info!("{:#?}", output.name);
        }
        exit(1);
    }

    // HASS setup
    log::info!("Creating the Websocket Client and Authenticate the session");
    let token = var("HASS_TOKEN").expect("Please set up the HASS_TOKEN env variable before running this");
    let host = var("HASS_HOST").unwrap_or("localhost".to_string());
    let port = var("HASS_PORT").unwrap_or("80".to_string()).parse::<u16>().expect("Please set up the HASS_PORT env variable before running this");
    let entity_id = var("HASS_ENTITY_ID").expect("Please set up the HASS_ENTITY_ID env variable before running this");

    let mut client = client::connect(&host, port).await?;
    client.auth_with_longlivedtoken(token.as_str()).await?;



    let output: WlOutput = if args.is_present("output") {
        output::get_wloutput(
            args.value_of("output").unwrap().trim().to_string(),
            output::get_all_outputs(&mut globals, &mut conn),
        )
    } else {
        output::get_all_outputs(&mut globals, &mut conn)
            .first()
            .unwrap()
            .wl_output
            .clone()
    };

    let duration = time::Duration::from_millis(pause_duration);
    let mut capturer = backend::setup_capture(&mut globals,&mut conn, &output).unwrap();
    let mut last_value = Hsl::from(0.0,0.0,0.0);
    loop {
        let frame_copy = backend::capture_output_frame(
            &mut globals,
            &mut conn,
            &output,
            &mut capturer,
        )?;
        let hsl = prominent_color::determine_prominent_color(frame_copy);
        if !hsl.eq(&last_value) {
            log::info!("Changing color to {:#?}", hsl);
    
            let value = Some(json!({
                "entity_id": entity_id,
                "hs_color": [hsl.get_hue(), hsl.get_saturation()],
                "transition": 1,
                "brightness_pct": hsl.get_lightness()
            }));
    
            client.call_service("light".to_string(), "turn_on".to_string(), value).await?;
            last_value = hsl.clone();
        }

        thread::sleep(duration);
    }
}