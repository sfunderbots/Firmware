To flash dongle run `opencd` first, then:

`cargo run --features dongle` To flash the mainboard as a dongle (connect to host over USB)
`cargo run --features robot` To flash the mainboard as a robot (connect to midplate)
