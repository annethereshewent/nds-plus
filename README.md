# NDS+

This is a DS emulator written in Rust! Binaries for Mac and Windows are now available. Go to releases and download the appropriate zip file for your operating system and unzip the files. To build in Linux, simply type `cargo build --release` and make sure you put the executable, `game_db.json`, and the `/freebios` directory all in the same directory.

## Getting Started

Extract the zip to a directory of your choice and open the executable from either the command line or GUI. The command line accepts the following arguments for Windows: `.\nds-plus.exe <path to rom file> [--start-bios]`

For MacOS, use `./nds-plus <path to rom file> [--start-bios]`

The optional `--start-bios` argument will boot up the firmware instead of performing a direct boot. In order to use the firmware, you will need to provide your own firmware and bios files.

To use your own files, simply copy the bios files to the root path of the app, and make sure they are named "bios7.bin", "bios9.bin", and "firmware.bin" for the bioses and firmware respectively.

### Web Client

To test the latest version of the emulator on web, go to https://nds-emulator.onrender.com. Copies of the bios and firmware files are now optional!

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

Hotkeys:

- *T*: Toggle control stick mode on/off (for Super Mario 64 DS)
- *F5*: Quick save state
- *F7*: Quick load state

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
- *L2 Button*: Quick save state
- *R2 Button*: Quick load state

## Screenshots

<img width="250" alt="Screenshot 2024-08-22 at 7 20 09 PM" src="https://github.com/user-attachments/assets/aee2e327-b552-4648-99fd-98be39994914">
<img width="250" alt="Screenshot 2024-08-22 at 7 20 54 PM" src="https://github.com/user-attachments/assets/8c2875df-d052-4d08-b1de-dd4126a1412e">
<img width="250" alt="Screenshot 2024-08-22 at 7 23 10 PM" src="https://github.com/user-attachments/assets/a5d50262-2383-4c5f-97a3-b46531fcfd9a">
<img width="250" alt="Screenshot 2024-08-22 at 7 24 06 PM" src="https://github.com/user-attachments/assets/db0f3eb3-02fd-46d3-b491-f22c575ab077">
<img width="250" alt="Screenshot 2024-08-22 at 7 43 05 PM" src="https://github.com/user-attachments/assets/1d41de7b-1089-4daa-943e-e5d79b6f9c6e">
<img width="250" alt="Screenshot 2024-08-22 at 7 39 35 PM" src="https://github.com/user-attachments/assets/43fb5b61-2037-4915-9cc6-5dfeacb3a62d">

## Special Thanks

Special thanks to the following individuals and organizations! Without their help this project wouldn't have gotten as far as it did.

- <a href="https://www.problemkaputt.de/gbatek.htm">GBATek</a> for being a really good resource on anything related to the Nintendo DS.
- The <a href="https://emudev.org/">EmuDev</a> discord server for providing a lot of helpful answers to any questions I had.
- <a href="https://github.com/abdllrhmanzedan">abdllzrhmanzedan</a> for providing all the designs for the iOS app. 

