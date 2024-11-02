---
outline: deep
---

<img class="icon" style="float:right;margin:20px;" alt="alt" src="/icons/robot-confused-outline.svg" width="120"/>

# Showcases

The following section will show you some examples as well as explanations on the main function for Hermes-Five.

<div style="clear:both"/>

::: info
Up-to-date and more detailed and numerous examples can be found in the
repository [examples folder](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples).
:::

## Various Hardware

- **[board/creation.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/board/creation.rs):**
  Shows how to instantiate a simple board using various protocols / transports layer.
- **[board/events.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/board/events.rs):** Shows
  how to react to board events.
- **[board/hardware.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/board/hardware.rs):**
  Shows how to access and control the hardware associated with a board: low level style!

## Generic devices

### Output

- **[output/digital.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/output/digital.rs):**
  Demonstrates how to control a digital output pin, regardless of the device type associated with it.
- **[output/pwm.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/output/pwm.rs):**
  Demonstrates how to control a pwm output pin, regardless of the device type associated with it.

### Input

- **[sensor/microwave.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/sensor/microwave.rs):**
  Demonstrates how to use a digital input pin to get a digital sensor type data.
- *
  *[sensor/potentiometer.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/sensor/potentiometer.rs):
  ** Demonstrates how to use an analog input pin to get an analog sensor type data.

## Various devices

### LED

- **[led/simple.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/led/simple.rs):**
  Demonstrates how to turn on/off a simple led.
- **[led/brightness.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/led/brightness.rs):**
  Demonstrates how to use a simple led with control over its brightness (requires a pwm pin).
- **[led/blink.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/led/blink.rs):** Demonstrates
  how to blink a simple led.
- **[led/pulse.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/led/pulse.rs):** Demonstrates
  how to pulse a simple led (requires a pwm pin).
- **[led/animate.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/led/animate.rs):**
  Demonstrates how to animate a led state.

### Servo

- **[servo/servo.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/servo/servo.rs):**
  Demonstrates how to use and control a servo.
- **[servo/sweep.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/servo/sweep.rs):**
  Demonstrates how to loop sweep a servo in a given range of motion.
- **[servo/animate.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/servo/animate.rs):**
  Demonstrates how to move a servo in an animated way (control of speed).
- **[servo/pca9685.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/servo/pca9685.rs):**
  Demonstrates how to move a servo via a PWM-driver like PCA9685.

### Button

- **[button/simple.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/button/simple.rs):**
  Demonstrates how to register a push button and retrieve its state using events.
- **[button/pullup.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/button/pullup.rs):**
  Demonstrates how to use a pullup type push button input device.
- **[button/inverted.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/button/inverted.rs):**
  Demonstrates how to use 'inverted' push buttons.

## Animation

- *
  *[animation/animation.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/animation/animation.rs):
  ** Demonstrates how to create and run a complex animation (with multiple devices, parts, repeating parts, etc.).
- *
  *[animation/multiple_animations.rs](https://github.com/dclause/hermes-five/tree/0.1.0/hermes-five/examples/animation/multiple_animations.rs):
  ** Demonstrates how to create multiple animations and run them at the same time.
