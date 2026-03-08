CC := arm-rockchip830-linux-uclibcgnueabihf/bin/arm-rockchip830-linux-uclibcgnueabihf-gcc
CFLAGS := -Wall -O2

hello: hello.c
	$(CC) $(CFLAGS) $^ -o $@

imu_eth: imu_eth.c
	$(CC) $(CFLAGS) $^ -o $@

all: hello imu_eth

clean:
	rm -f hello imu_eth
