---
outline: deep
---

# Troubleshooting

<img class="icon" style="float:right" alt="alt" src="/icons/robot-dead-outline.svg" width="100"/>

This section lists a few topics that may help you troubleshoot issues with Hermes-Five.

If you can't find a solution to your problem on this page,
consider [opening an issue](https://github.com/dclause/hermes-five/issues) for help.

## Protocol error

This error type appears in cases of issues related to board communication.

::: details Error: _Task failed: "Protocol error: The filename, directory name, or volume label syntax is incorrect.""_
When running the basic sample code from [Getting Started](/getting-started) section, the program ends right away with
the error:   
**_Task failed: "Protocol error: The filename, directory name, or volume label syntax is incorrect."_**

1. Check that your board is actually connected to your computer.
2. The port used may not be properly auto-detected. In that case, use the custom protocol syntax to specify the
   appropriate port name when creating your board.

```
let board = Board::from(SerialProtocol::new("COM4")).open();
```

:::

::: details Error: _Task failed: "Protocol error: Operation timed out."_
When running the basic sample code from [Getting Started](/getting-started) section, the program ends right away with
the error:   
**_Task failed: "Protocol error: Operation timed out."_**

1. Check that your board has the proper and
   latest [StandardFirmataPlus.ino](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino)
   Arduino sketch uploaded.
   :::
