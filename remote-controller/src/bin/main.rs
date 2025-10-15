#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::rng::Rng;
use esp_hal::time::Duration;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{main, time};
use esp_println as _;
use esp_wifi::esp_now::{PeerInfo, BROADCAST_ADDRESS};
use esp_wifi::init;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

enum Actions {
    Move,
    Break,
    Accel,
    Halt,
}

fn action_to_msg(action: &Actions) -> &'static [u8] {
    match action {
        Actions::Move => b"Move",
        Actions::Break => b"Break",
        Actions::Accel => b"Accel",
        Actions::Halt => b"Halt",
    }
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    esp_alloc::heap_allocator!(size: 64 * 1024);
    let actions = [Actions::Move, Actions::Break, Actions::Accel, Actions::Halt];

    info!("ESP-NOW Émetteur démarré");
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let esp_wifi_ctrl = init(timg0.timer0, Rng::new(peripherals.RNG)).unwrap();

    let wifi = peripherals.WIFI;
    let (mut controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, wifi).unwrap();
    controller.set_mode(esp_wifi::wifi::WifiMode::Sta).unwrap();
    controller.start().unwrap();

    let mut esp_now = interfaces.esp_now;

    info!("esp-now version {}", esp_now.version().unwrap());

    esp_now.set_channel(11).unwrap();

    info!("ESP-NOW initialisé - Recherche du robot...");

    let mut robot_address: Option<[u8; 6]> = None;
    let mut next_discovery_broadcast = time::Instant::now();
    let delay = Delay::new();
    let mut action_index = 0;

    loop {
        let r = esp_now.receive();
        if let Some(r) = r {
            let src = r.info.src_address;
            let dst = r.info.dst_address;
            let message_str = core::str::from_utf8(r.data()).unwrap_or("<invalid UTF-8>");

            info!(
                "Message reçu de {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}: '{}'",
                src[0], src[1], src[2], src[3], src[4], src[5], message_str
            );

            if dst != BROADCAST_ADDRESS && message_str == "CONNECTED" {
                info!("Robot découvert !");
                robot_address = Some(src);
                led.toggle();
            }

            if dst == BROADCAST_ADDRESS && robot_address.is_none() {
                info!("Robot découvert !");
                robot_address = Some(src);
                led.toggle();

                if !esp_now.peer_exists(&src) {
                    esp_now
                        .add_peer(PeerInfo {
                            interface: esp_wifi::esp_now::EspNowWifiInterface::Sta,
                            peer_address: src,
                            lmk: None,
                            channel: None,
                            encrypt: false,
                        })
                        .unwrap();
                }

                let status = esp_now.send(&src, b"CONNECTED").unwrap().wait();
                info!("Confirmation envoyée au robot: {:?}", status);
            }
        }

        if robot_address.is_none() {
            if time::Instant::now() >= next_discovery_broadcast {
                next_discovery_broadcast = time::Instant::now() + Duration::from_secs(2);
                let status = esp_now
                    .send(&BROADCAST_ADDRESS, b"CONTROLLER_PING")
                    .unwrap()
                    .wait();
                info!("Broadcast envoyé: {:?}", status.unwrap());
            }
        } else {
            if action_index == actions.len() {
                action_index = 0;
            }

            if let Some(address) = robot_address {
                let _ = esp_now
                    .send(&address, action_to_msg(&actions[action_index]))
                    .unwrap();
                action_index += 1;
            }
            delay.delay_millis(5000);
        }
    }
}
