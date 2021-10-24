# Govee Collector

Microservice for collecting and processing data from Govee bluetooth hygrometers.

## Why?

Govee provides a smartphone app, that is good enough for most people,
but it requires creating an account and storing data in the cloud to
access the most interesting features.
As a privacy ~~freak~~ advocate, I'm planning to achieve the same functionality
by collecting and processing the data on a local Raspberry Pi
(or any other 24/7 Linux or macOS machine).

## Supported devices

* [Govee H5075](https://www.amazon.com/dp/B07Y36FWTT)

## Supported operating systems

The service officially supports macOS and Linux, but since all tools and libraries
that I use are cross-platform, it could work on other OS's too – give it a try.

### macOS permissions note

To use Bluetooth on macOS Big Sur (11) or later, you need to either package your
binary into an application bundle with an `Info.plist` including
`NSBluetoothAlwaysUsageDescription`, or (for a command-line application such as
the examples included with `btleplug`) enable the Bluetooth permission for your
terminal. You can do the latter by going to _System Preferences_ → _Security &
Privacy_ → _Privacy_ → _Bluetooth_, clicking the '+' button, and selecting
'Terminal' (or iTerm or whichever terminal application you use).
