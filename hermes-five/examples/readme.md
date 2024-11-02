The available examples are:

# Various Hardware

- **board/creation.rs:** Shows how to instantiate a simple board using various protocols / transports layer.
- **board/events.rs:** Shows how to react to board events.
- **board/hardware.rs:** Shows how to access and control the hardware associated with a board: low level style!

# Generic devices

## Output

- **output/digital.ts:** Demonstrates how to control a digital output pin, regardless of the device type associated with
  it.
- **output/pwm.rs:** Demonstrates how to control a pwm output pin, regardless of the device type associated with it.

## Input

- **sensor/microwave.rs:** Demonstrates how to use a digital input pin to get a digital sensor type data.
- **sensor/potentiometer.rs:** Demonstrates how to use an analog input pin to get an analog sensor type data.

# Various devices

## LED

- **led/simple.rs:** Demonstrates how to turn on/off a simple led.
- **led/brightness.rs:** Demonstrates how to use a simple led with control over its brightness (requires a pwm pin).
- **led/blink.rs:** Demonstrates how to blink a simple led.
- **led/pulse.rs:** Demonstrates how to pulse a simple led (requires a pwm pin).
- **led/animate.rs:** Demonstrates how to animate a led state.

## Servo

- **servo/servo.rs:** Demonstrates how to use and control a servo.
- **servo/sweep.rs:** Demonstrates how to loop sweep a servo in a given range of motion.
- **servo/animate.rs:** Demonstrates how to move a servo in an animated way (control of speed).
- **servo/pca9685.rs:** Demonstrates how to move a servo via a PWM-driver like PCA9685.

## Button

- **button/simple.rs:** Demonstrates how to register a push button and retrieve its state using events.
- **button/pullup.rs:** Demonstrates how to use a pullup type push button input device.
- **button/inverted.rs:** Demonstrates how to use 'inverted' push buttons.

# Animation

- **animation/animation.rs:** Demonstrates how to create and run a complex animation (with multiple devices, parts,
  repeating parts, etc.).
- **animation/multiple_animations.rs:** Demonstrates how to create multiple animations and run them at the same time.
