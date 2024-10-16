# NDS+

This is a DS emulator written in Rust! Binaries for Mac and Windows are now available. Go to releases and download the appropriate zip file for your operating system and unzip the files. To build in Linux, simply type `cargo build --release` and make sure you put the executable, `game_db.json`, and the `/freebios` directory all in the same directory.

## Getting Started

### Windows Users

Extract the zip to a directory of your choice and open the executable from either the command line or GUI. The command line accepts the following arguments: `.\nds-plus.exe <path to rom file> [--start-bios]`

The `--start-bios` argument will boot up the firmware instead of performing a direct boot. In order to use the firmware, you will need to provide your own firmware and bios files.

Please see the section below on how to use your own files.

### Mac Users

Simply extract the zip and open the .app file and everything should work out of the box. Please see section "Using your own binary files" for how to use your own provided binary files.

To open from the command line, simply run `./NDS\ Plus.app/Contents/MacOS/nds-plus <path to rom file> [--start-bios]`.

### Web Client

To test the latest version of the emulator on web, go to https://nds-emulator.onrender.com. You will need copies of the ARM7 and ARM9 BIOSes as well as the DS firmware.

## Using your own binary files

Make sure that your binary files are named `bios7.bin` for the ARM7 bios, `bios9.bin` for the ARM9 bios, and `firmware.bin` for the firmware. 

### Windows 

For Windows, simply put the files in the same directory as the executable.

### Mac

For Mac, you will need to open the .app file by right clicking on it, hit "Show Package Contents," then go to the `Contents/MacOS` directory. Copy the files over and you should be all set!

## Features

- Support for web, desktop, and iOS (coming soon)
- Ability to use control stick in Super Mario 64 DS
- Save management on the web and iOS clients: upload, download and delete saves
- Cloud saves are now available! Store saves in Google drive for use anywhere on both desktop, web, and iOS.
- Support for microphone on desktop, web, and iOS
- Save states on desktop and iOS

## TODO

- Texture/rendering issues
- CPU bugs
- iOS app (almost complete!)
- Debugging tools

## Controls

Keyboard:

- *Up*: W Key
- *Down*: S Key
- *Left*: A Key
- *Right*: D Key
- *A Button*: K Key
- *B Button*: J Key
- *Y Button*: N Key
- *X Button*: M Key
- *L Button*: C Key
- *R Button*: V Key
- *Select*: Tab
- *Start*: Return
- *T key*: Toggle control stick mode on/off (For Super Mario 64 DS)

Joypad (tested on PS5 controller, should be similar on Xbox/other similar controllers)

- *Directions*: Control pad
- *A Button*: Circle
- *B Button*: Cross
- *Y Button*: Square
- *X Button*: Triangle
- *L Button*: L1
- *R BUtton*: R1
- *Select*: Select
- *Start*: Start
- *R3 Button*: Toggle control stick mode on/off (For Super Mario 64 DS)

## Screenshots

<img width="250" alt="Screenshot 2024-08-22 at 7 20 09 PM" src="https://github.com/user-attachments/assets/aee2e327-b552-4648-99fd-98be39994914">
<img width="250" alt="Screenshot 2024-08-22 at 7 20 54 PM" src="https://github.com/user-attachments/assets/8c2875df-d052-4d08-b1de-dd4126a1412e">
<img width="250" alt="Screenshot 2024-08-22 at 7 23 10 PM" src="https://github.com/user-attachments/assets/a5d50262-2383-4c5f-97a3-b46531fcfd9a">
<img width="250" alt="Screenshot 2024-08-22 at 7 24 06 PM" src="https://github.com/user-attachments/assets/db0f3eb3-02fd-46d3-b491-f22c575ab077">
<img width="250" alt="Screenshot 2024-08-22 at 7 43 05 PM" src="https://github.com/user-attachments/assets/1d41de7b-1089-4daa-943e-e5d79b6f9c6e">
<img width="250" alt="Screenshot 2024-08-22 at 7 39 35 PM" src="https://github.com/user-attachments/assets/43fb5b61-2037-4915-9cc6-5dfeacb3a62d">



