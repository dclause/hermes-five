---
outline: deep
---

# Troubleshooting

<img class="icon" style="float:right;margin:20px;" alt="alt" src="/icons/robot-dead-outline.svg" width="150"/>

This section lists a few topics that may help you troubleshoot issues with Hermes-Five.

If you can't find a solution to your problem on this page,
consider [opening an issue](https://github.com/dclause/hermes-five/issues) for help.

## Protocol error

This error type appears in cases of issues related to board communication.

::: details Error: _Task failed: "Protocol error: The filename, directory name, or volume label syntax is incorrect."_
When running the basic sample code from [Getting Started](./getting-started) section, the program ends right away with
the error:   
**_Task failed: "Protocol error: The filename, directory name, or volume label syntax is incorrect."_**
1. Check that your board is actually connected to your computer.
2. The port used may not be properly auto-detected. In that case, use the custom protocol syntax to specify the
   appropriate port name when creating your board.
```rust
let board = Board::from(SerialProtocol::new("COM4")).open();
```
:::

::: details Error: _Task failed: "Protocol error: Operation timed out."_
When running the basic sample code from [Getting Started](./getting-started) section, the program ends right away with
the error:   
**_Task failed: "Protocol error: Operation timed out."_**
1. Check that your board has the proper and latest [StandardFirmataPlus.ino](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino) Arduino sketch uploaded.
:::



## Hardware error

This error type appears in cases of issues related to device <-> hardware communication.

::: details Error: _HardwareError { source: UnknownPin { pin: Id(13) } }_
- Check your pin number: it may not exist for you board.
- If it does: did you properly opened the board first ? Devices must be registered in the ready event (see [Concepts & Overview page](./concepts#wait-until-its-ready))
:::

::: details Error: _Hardware error: Pin (8) not compatible with mode (ANALOG) - try to set pin mode._
The error message is pretty self explicit: your pin (8 here) is not compatible with what you are trying to do (read an ANALOG value here).
Change the pin for a compatible one.<br/>
The folowing code will display you all pins and there compatibility:
```rust
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::io::IO;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();
    board.on(BoardEvent::OnReady, |mut board: Board| async move {
        println!("Pins {:#?}", board.get_io().read().pins);
        Ok(())
    });
}
```
:::
