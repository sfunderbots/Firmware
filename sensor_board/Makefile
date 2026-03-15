CC := arm-rockchip830-linux-uclibcgnueabihf/bin/arm-rockchip830-linux-uclibcgnueabihf-gcc
CFLAGS := -Wall -O2
TARGET := imu_eth
SRC := imu_eth.c

.PHONY: all clean docker-build

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) $^ -o $@

docker-build:
	./docker-build.sh

clean:
	rm -f $(TARGET)
