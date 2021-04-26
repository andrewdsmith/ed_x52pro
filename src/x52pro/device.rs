use crate::game::StatusLevel;
use crate::x52pro::direct_output::DirectOutput;
use std::collections::HashMap;
use std::time::SystemTime;

pub const ALERT_FLASH_MILLISECONDS: u128 = 500;

/// An instance of an interface to a Saitek X52 Pro Flight HOTAS flight
/// controller device.
pub struct Device {
    direct_output: DirectOutput,
    input_state_levels: HashMap<Input, StatusLevel>,
    reference_time: SystemTime,
}

impl Device {
    /// Returns a new instance the the device interface. Panics if the
    /// underlying DirectOutput instance cannot be loaded.
    pub fn new() -> Self {
        let mut direct_output = DirectOutput::load();
        direct_output.initialize();
        direct_output.enumerate();
        direct_output.add_page();

        Device {
            direct_output: direct_output,
            input_state_levels: HashMap::new(),
            reference_time: SystemTime::now(),
        }
    }

    /// Sets the given input to specified status level. The LED for the input
    /// is looked up, as is the LED state for the status level.
    pub fn set_input_status_level(&mut self, input: Input, status_level: StatusLevel) {
        self.set_led_from_input_and_status_level(&input, &status_level);
        self.input_state_levels.insert(input, status_level);
    }

    /// Set the given LED to the specified state.
    pub fn set_led_state(&self, led: LED, state: LEDState) {
        let (red_led_id, green_led_id) = match led {
            LED::Clutch => (17, 18),
            LED::FireA => (1, 2),
            LED::FireB => (3, 4),
            LED::FireD => (5, 6),
            LED::FireE => (7, 8),
            LED::T1T2 => (9, 10),
            LED::T3T4 => (11, 12),
            LED::T5T6 => (13, 14),
        };

        let (red_led_state, green_led_state) = match state {
            LEDState::Red => (true, false),
            LEDState::Amber => (true, true),
            LEDState::Green => (false, true),
        };

        self.direct_output.set_led(red_led_id, red_led_state);
        self.direct_output.set_led(green_led_id, green_led_state);
    }

    /// Updates LEDs that have a state that is animated, e.g. flashing. This
    /// needs to be called frequently for proper animation.
    //
    // Ideally the device would manage its own threading for animation but
    // this would require state updates to be communicated asynchronously.
    pub fn update_animated_leds(&self) {
        for (input, status_level) in &self.input_state_levels {
            if *status_level == StatusLevel::Alert {
                self.set_led_from_input_and_status_level(input, status_level);
            }
        }
    }

    fn set_led_from_input_and_status_level(&self, input: &Input, status_level: &StatusLevel) {
        self.set_led_state(
            led_for_input(input),
            led_state_for_status_level(status_level, self.reference_time),
        );
    }
}

/// Supported input buttons or axes on the device.
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Input {
    Clutch,
    FireA,
    FireB,
    FireD,
    FireE,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
}

/// Controllable LEDs on the devive.
#[derive(Debug, PartialEq)]
pub enum LED {
    Clutch,
    FireA,
    FireB,
    FireD,
    FireE,
    T1T2,
    T3T4,
    T5T6,
}

/// Available states for LEDs on the device.
#[derive(Debug, PartialEq)]
pub enum LEDState {
    Red,
    Amber,
    Green,
}

/// Returns the LED that corresponds to a given input. Note that in some cases,
/// specifically the T buttons, multiple inputs share an LED.
fn led_for_input(input: &Input) -> LED {
    match input {
        Input::Clutch => LED::Clutch,
        Input::FireA => LED::FireA,
        Input::FireB => LED::FireB,
        Input::FireD => LED::FireD,
        Input::FireE => LED::FireE,
        Input::T1 => LED::T1T2,
        Input::T2 => LED::T1T2,
        Input::T3 => LED::T3T4,
        Input::T4 => LED::T3T4,
        Input::T5 => LED::T5T6,
        Input::T6 => LED::T5T6,
    }
}

/// Returns the LED state that corrsponds to a given status level.
fn led_state_for_status_level(status_level: &StatusLevel, reference_time: SystemTime) -> LEDState {
    match status_level {
        StatusLevel::Inactive => LEDState::Green,
        StatusLevel::Active => LEDState::Amber,
        StatusLevel::Blocked => LEDState::Red,
        StatusLevel::Alert => {
            let millis = reference_time.elapsed().unwrap().as_millis();
            if (millis / ALERT_FLASH_MILLISECONDS) & 1 == 0 {
                LEDState::Red
            } else {
                LEDState::Amber
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_to_led_permutations() {
        assert_led_for_input(Input::Clutch, LED::Clutch);
        assert_led_for_input(Input::FireA, LED::FireA);
        assert_led_for_input(Input::FireB, LED::FireB);
        assert_led_for_input(Input::FireD, LED::FireD);
        assert_led_for_input(Input::FireE, LED::FireE);
        assert_led_for_input(Input::T1, LED::T1T2);
        assert_led_for_input(Input::T2, LED::T1T2);
        assert_led_for_input(Input::T3, LED::T3T4);
        assert_led_for_input(Input::T4, LED::T3T4);
        assert_led_for_input(Input::T5, LED::T5T6);
        assert_led_for_input(Input::T6, LED::T5T6);
    }

    fn assert_led_for_input(input: Input, led: LED) {
        assert_eq!(led_for_input(&input), led);
    }

    #[test]
    fn status_level_for_led_state_permutations() {
        assert_led_state_for_status_level(StatusLevel::Inactive, LEDState::Green);
        assert_led_state_for_status_level(StatusLevel::Active, LEDState::Amber);
        assert_led_state_for_status_level(StatusLevel::Blocked, LEDState::Red);
    }

    fn assert_led_state_for_status_level(status_level: StatusLevel, led_state: LEDState) {
        assert_eq!(
            led_state_for_status_level(&status_level, SystemTime::now()),
            led_state,
        );
    }
}
