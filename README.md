# Govee Collector

Microservice for collecting and processing data from Govee bluetooth hygrometers.

## What?

Immediately after launch the service starts listening for BLE advertisement data
from the devices defined in `~/.govee_devices.toml`.
The collected data is accessible via [gRPC](https://grpc.io).

Proto file: [proto/service.proto](./proto/service.proto).

## How to use?

1. Checkout the repository:

   ```shell
   git clone --recurse-submodules https://github.com/Samarkin/govee_collector
   cd govee_collector
   ```
2. Prepare `.govee_devices.toml` in your home directory. 

   For example:
   ```toml
   [GVH5075_6A19]
   friendly_name = "Living Room"
   ```
   
3. Build and run:

   ```shell
   cargo run
   ```

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

## FAQ

### macOS permissions issue

To use Bluetooth on macOS Big Sur (11) or later, you need to either package your
binary into an application bundle with an `Info.plist` including
`NSBluetoothAlwaysUsageDescription`, or (for a command-line application such as
the examples included with `btleplug`) enable the Bluetooth permission for your
terminal. You can do the latter by going to _System Preferences_ → _Security &
Privacy_ → _Privacy_ → _Bluetooth_, clicking the '+' button, and selecting
'Terminal' (or iTerm or whichever terminal application you use).

### Failed to find the protoc binary

On macOS, you can use [HomeBrew](https://brew.sh):
```shell
brew install protobuf
```

On Raspbian, similarly:
```shell
sudo apt-get install protobuf-compiler
```

Also, you can download the latest version of `protoc` here:
https://github.com/protocolbuffers/protobuf/releases
