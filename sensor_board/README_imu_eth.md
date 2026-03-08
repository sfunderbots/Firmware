# IMU to Raw Ethernet Publisher (Luckfox Pico Pro)

This program configures an LSM6DSOX over Linux `i2c-dev`, polls one sample, timestamps it, and sends one UDP packet per sample.

## Build

```sh
cd /Users/vikrambhat/Firmware/sensor_board
make imu_eth
```

## Run

```sh
./imu_eth [iface] [dst_ip] [dst_port] [i2c_dev] [i2c_addr_hex] [hz]
```

Defaults:

- `iface=eth0`
- `dst_ip=255.255.255.255`
- `dst_port=5005`
- `i2c_dev=/dev/i2c-1`
- `i2c_addr=0x6A`
- `hz=100`

Example:

```sh
./imu_eth eth0 255.255.255.255 5005 /dev/i2c-1 0x6A 100
```

## UDP Payload

- Payload struct (`imu_frame_v1`) is big-endian integer fields.
- Includes:
  - magic/version/sequence
  - monotonic and realtime timestamps (`ns`)
  - raw temp/gyro/accel
  - scaled values:
    - temp: `mdegC`
    - gyro: `mdps`
    - accel: `um/s^2`

Notes:

- Current IMU config in code:
  - accel: 104 Hz, +/-4 g
  - gyro: 104 Hz, +/-2000 dps
- If your board straps `SA0` high, I2C address may be `0x6B`.

## Linux Receiver + Web Plot

Script: `imu_eth_viewer.py`

Install dependency:

```sh
python3 -m pip install flask
```

Run:

```sh
python3 imu_eth_viewer.py --listen-host 0.0.0.0 --udp-port 5005 --host 0.0.0.0 --port 5000
```

Then open from any computer that can reach that Linux box:

```txt
http://<linux-box-ip>:5000/
```

The page plots live gyro, accel, and temperature from incoming UDP packets.
