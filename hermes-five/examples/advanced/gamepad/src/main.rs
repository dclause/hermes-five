use std::time::{Duration, SystemTime};
use gilrs::EventType::{AxisChanged, ButtonChanged};
/// Gamepad controller example with a servo motor.
///
/// This example shows how to read the left stick X axis of a Gamepad controller such as a PS4 controller
/// to control a servo motor attached to a board.
///
/// The example uses the `gilrs` crate to read the controller events and the `hermes_five` crate to control
/// the servo motor.
use gilrs::{Axis, Button, Event, Gilrs};
use hermes_five::devices::{Led, Servo};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;
use hermes_five::utils::Range;

// ######################
// DEMO CONFIGURATION
// For demonstration purpose, the configuration as left basic.
// No acceleration or inertia considered.
const UPDATE_INTERVAL_MS: u64 = 20;                         // ~50 Hz
const IDLE_DELAY: u64 = 5000;                               // Time after which the servo and led are reset.
// servo
const SERVO_PIN: u8 = 9;                                    // Board pin
const SERVO_REST_POSITION: f32 = 90.0;                      // center
const SERVO_RANGE: Range<u16> = Range { start: 0, end: 180 };
const SERVO_ACCELERATION: f32 = 200.0;                      // °/s²
const SERVO_INVERSION: i8 = 1;                              // 1: normal, -1: inverted
 // led
const LED_PIN: u8 = 13;                                     // Embedded arduino led by default
const LED_SPEED: Range<u16> = Range {start: 50, end: 500 };
const LED_DELAY_ACCELERATION: f32 = 100.0;                   // ms/s²

#[hermes_five::runtime]
async fn main() {

    // Initiate a board on auto-detected port.
    // Don't forget to flash it first with
    // https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino
    let board = Board::start().unwrap();
    board.on(BoardEvent::OnReady, |board: Board| async move {

        // Init gamepad.
        let mut gilrs = Gilrs::new().unwrap();

        // Initialize servo
        let mut servo = Servo::new(&board, SERVO_PIN, SERVO_REST_POSITION as u16)?.set_range(SERVO_RANGE);
        let mut servo_req_position = SERVO_REST_POSITION as f32;
        let mut servo_acceleration = 0.0;
        let mut servo_last_activity_time = SystemTime::now();
        let mut servo_need_idle = true;

        // Initialize led
        let mut led = Led::new(&board, LED_PIN, false)?;
        let mut led_delay = ((LED_SPEED.end - LED_SPEED.start) / 2) as f32;
        let mut led_req_delay = led_delay.clone();
        let mut led_acceleration = 0.0;
        led.blink(led_delay as u64);

        println!("Board ready, PS4 controller ready, starting main loop...");

        loop {
            // Examine new events
            while let Some(Event { event, time, .. }) = gilrs.next_event() {
                // Set the servo and led accelerations accordingly.
                match event {
                    AxisChanged(Axis::LeftStickX, acceleration, _) => {
                        servo_acceleration = acceleration * SERVO_ACCELERATION * SERVO_INVERSION as f32;
                        servo_last_activity_time  =  time;
                        servo_need_idle = acceleration == 0.0;
                    },
                    ButtonChanged(Button::LeftTrigger2, acceleration, _) => {
                        led_acceleration = acceleration * -LED_DELAY_ACCELERATION;
                    },
                    ButtonChanged(Button::RightTrigger2, acceleration, _) => {
                        led_acceleration = acceleration * LED_DELAY_ACCELERATION;
                    },
                    _ => {}
                }
            }

            // Adjust servo position: auto-back to center after IDLE_DELAY ms.
            if servo_need_idle && servo_last_activity_time.elapsed().unwrap() > Duration::from_millis(IDLE_DELAY) {
                servo.to(SERVO_REST_POSITION as u16)?;
                servo_req_position = SERVO_REST_POSITION;
                servo_need_idle = false;
            } else {
                servo_req_position = (servo_req_position
                    + servo_acceleration * (UPDATE_INTERVAL_MS as f32 / 1_000.0))
                    .clamp(SERVO_RANGE.start as f32, SERVO_RANGE.end as f32);
                if (servo_req_position - servo.get_position() as f32).abs() > 1.0 {
                    servo.to(servo_req_position as u16)?;
                }
            }

            // Adjust led delay
            led_req_delay = (led_req_delay
                + led_acceleration * (UPDATE_INTERVAL_MS as f32 / 1_000.0))
                .clamp(LED_SPEED.start as f32, LED_SPEED.end as f32);
            if (led_req_delay - led_delay).abs() > 10.0 {
                led.blink(led_req_delay as u64);
                led_delay = led_req_delay;
            }

            // Pause
            pause!(UPDATE_INTERVAL_MS);
        }

        #[allow(unreachable_code)]
        Ok(())
    });
}
