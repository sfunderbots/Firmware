#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/i2c-dev.h>
#include <net/if.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <time.h>
#include <unistd.h>

#define LSM6DSOX_I2C_ADDR_DEFAULT 0x6A
#define LSM6DSOX_REG_WHO_AM_I 0x0F
#define LSM6DSOX_WHO_AM_I_VAL 0x6C
#define LSM6DSOX_REG_CTRL1_XL 0x10
#define LSM6DSOX_REG_CTRL2_G 0x11
#define LSM6DSOX_REG_CTRL3_C 0x12
#define LSM6DSOX_REG_OUT_TEMP_L 0x20

#define IMU_MAGIC 0x494D5530u
#define IMU_VERSION 1
#define UDP_PORT_DEFAULT 5005

static volatile sig_atomic_t keep_running = 1;

static uint64_t htonll_u64(uint64_t v)
{
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
    return (((uint64_t)htonl((uint32_t)(v & 0xFFFFFFFFu))) << 32) |
           htonl((uint32_t)(v >> 32));
#else
    return v;
#endif
}

static int i2c_write_reg(int fd, uint8_t reg, uint8_t value)
{
    uint8_t buf[2] = {reg, value};
    ssize_t w = write(fd, buf, sizeof(buf));
    return (w == (ssize_t)sizeof(buf)) ? 0 : -1;
}

static int i2c_read_regs(int fd, uint8_t start_reg, uint8_t *buf, size_t len)
{
    ssize_t w = write(fd, &start_reg, 1);
    if (w != 1) {
        return -1;
    }
    ssize_t r = read(fd, buf, len);
    return (r == (ssize_t)len) ? 0 : -1;
}

static int16_t le16_to_i16(const uint8_t *p)
{
    return (int16_t)((uint16_t)p[0] | ((uint16_t)p[1] << 8));
}

static void handle_sigint(int sig)
{
    (void)sig;
    keep_running = 0;
}

struct __attribute__((packed)) imu_frame_v1 {
    uint32_t magic_be;
    uint8_t version;
    uint8_t reserved;
    uint16_t payload_len_be;
    uint32_t seq_be;
    uint64_t mono_ns_be;
    uint64_t real_ns_be;
    int16_t temp_raw_be;
    int16_t gyro_raw_be[3];
    int16_t accel_raw_be[3];
    int32_t temp_mdegc_be;
    int32_t gyro_mdps_be[3];
    int32_t accel_ums2_be[3];
};

static void usage(const char *prog)
{
    fprintf(stderr,
            "Usage: %s [iface] [dst_ip] [dst_port] [i2c_dev] [i2c_addr_hex] [hz]\n"
            "Defaults:\n"
            "  iface=eth0 dst_ip=255.255.255.255 dst_port=5005 i2c_dev=/dev/i2c-1 i2c_addr=0x6A hz=100\n",
            prog);
}

int main(int argc, char **argv)
{
    const char *iface = (argc > 1) ? argv[1] : "eth0";
    const char *dst_ip = (argc > 2) ? argv[2] : "255.255.255.255";
    int dst_port = (argc > 3) ? atoi(argv[3]) : UDP_PORT_DEFAULT;
    const char *i2c_dev = (argc > 4) ? argv[4] : "/dev/i2c-1";
    int i2c_addr = (argc > 5) ? (int)strtol(argv[5], NULL, 0) : LSM6DSOX_I2C_ADDR_DEFAULT;
    int hz = (argc > 6) ? atoi(argv[6]) : 100;
    int i2c_fd = -1;
    int udp_fd = -1;
    uint32_t seq = 0;
    struct sockaddr_in dst_addr;

    if (hz <= 0 || hz > 2000) {
        fprintf(stderr, "Invalid poll rate: %d Hz\n", hz);
        return 1;
    }
    if (dst_port <= 0 || dst_port > 65535) {
        fprintf(stderr, "Invalid dst_port: %d\n", dst_port);
        return 1;
    }

    signal(SIGINT, handle_sigint);
    signal(SIGTERM, handle_sigint);

    i2c_fd = open(i2c_dev, O_RDWR);
    if (i2c_fd < 0) {
        perror("open i2c");
        return 1;
    }
    if (ioctl(i2c_fd, I2C_SLAVE, i2c_addr) < 0) {
        perror("ioctl I2C_SLAVE");
        close(i2c_fd);
        return 1;
    }

    uint8_t who = 0;
    if (i2c_read_regs(i2c_fd, LSM6DSOX_REG_WHO_AM_I, &who, 1) != 0) {
        perror("read WHO_AM_I");
        close(i2c_fd);
        return 1;
    }
    if (who != LSM6DSOX_WHO_AM_I_VAL) {
        fprintf(stderr, "Unexpected WHO_AM_I: 0x%02X (expected 0x%02X)\n", who, LSM6DSOX_WHO_AM_I_VAL);
        close(i2c_fd);
        return 1;
    }

    if (i2c_write_reg(i2c_fd, LSM6DSOX_REG_CTRL3_C, 0x44) != 0 ||
        i2c_write_reg(i2c_fd, LSM6DSOX_REG_CTRL1_XL, 0x48) != 0 ||
        i2c_write_reg(i2c_fd, LSM6DSOX_REG_CTRL2_G, 0x4C) != 0) {
        perror("init IMU regs");
        close(i2c_fd);
        return 1;
    }

    udp_fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (udp_fd < 0) {
        perror("socket UDP");
        close(i2c_fd);
        return 1;
    }

    int broadcast = 1;
    if (setsockopt(udp_fd, SOL_SOCKET, SO_BROADCAST, &broadcast, sizeof(broadcast)) != 0) {
        perror("setsockopt SO_BROADCAST");
        close(udp_fd);
        close(i2c_fd);
        return 1;
    }

#ifdef SO_BINDTODEVICE
    if (setsockopt(udp_fd, SOL_SOCKET, SO_BINDTODEVICE, iface, strlen(iface)) != 0) {
        fprintf(stderr, "Warning: SO_BINDTODEVICE(%s) failed: %s\n", iface, strerror(errno));
    }
#endif

    memset(&dst_addr, 0, sizeof(dst_addr));
    dst_addr.sin_family = AF_INET;
    dst_addr.sin_port = htons((uint16_t)dst_port);
    if (inet_pton(AF_INET, dst_ip, &dst_addr.sin_addr) != 1) {
        fprintf(stderr, "Invalid dst_ip: %s\n", dst_ip);
        close(udp_fd);
        close(i2c_fd);
        return 1;
    }

    struct timespec period;
    period.tv_sec = 0;
    period.tv_nsec = 1000000000L / hz;

    printf("Publishing IMU UDP on %s to %s:%d @ %d Hz, i2c=%s addr=0x%02X\n",
           iface, dst_ip, dst_port, hz, i2c_dev, i2c_addr);

    while (keep_running) {
        uint8_t raw[14];
        struct timespec mono_ts;
        struct timespec real_ts;
        struct imu_frame_v1 frame;

        if (clock_gettime(CLOCK_MONOTONIC, &mono_ts) != 0 || clock_gettime(CLOCK_REALTIME, &real_ts) != 0) {
            perror("clock_gettime");
            break;
        }
        if (i2c_read_regs(i2c_fd, LSM6DSOX_REG_OUT_TEMP_L, raw, sizeof(raw)) != 0) {
            perror("read sensor");
            break;
        }

        int16_t temp_raw = le16_to_i16(&raw[0]);
        int16_t gx_raw = le16_to_i16(&raw[2]);
        int16_t gy_raw = le16_to_i16(&raw[4]);
        int16_t gz_raw = le16_to_i16(&raw[6]);
        int16_t ax_raw = le16_to_i16(&raw[8]);
        int16_t ay_raw = le16_to_i16(&raw[10]);
        int16_t az_raw = le16_to_i16(&raw[12]);

        int32_t temp_mdegc = 25000 + ((int32_t)temp_raw * 1000) / 256;
        int32_t gx_mdps = (int32_t)gx_raw * 70;
        int32_t gy_mdps = (int32_t)gy_raw * 70;
        int32_t gz_mdps = (int32_t)gz_raw * 70;
        int32_t ax_ums2 = (int32_t)((int64_t)ax_raw * 1196);
        int32_t ay_ums2 = (int32_t)((int64_t)ay_raw * 1196);
        int32_t az_ums2 = (int32_t)((int64_t)az_raw * 1196);

        uint64_t mono_ns = (uint64_t)mono_ts.tv_sec * 1000000000ull + (uint64_t)mono_ts.tv_nsec;
        uint64_t real_ns = (uint64_t)real_ts.tv_sec * 1000000000ull + (uint64_t)real_ts.tv_nsec;

        memset(&frame, 0, sizeof(frame));
        frame.magic_be = htonl(IMU_MAGIC);
        frame.version = IMU_VERSION;
        frame.reserved = 0;
        frame.payload_len_be = htons((uint16_t)sizeof(struct imu_frame_v1));
        frame.seq_be = htonl(seq++);
        frame.mono_ns_be = htonll_u64(mono_ns);
        frame.real_ns_be = htonll_u64(real_ns);
        frame.temp_raw_be = htons((uint16_t)temp_raw);
        frame.gyro_raw_be[0] = htons((uint16_t)gx_raw);
        frame.gyro_raw_be[1] = htons((uint16_t)gy_raw);
        frame.gyro_raw_be[2] = htons((uint16_t)gz_raw);
        frame.accel_raw_be[0] = htons((uint16_t)ax_raw);
        frame.accel_raw_be[1] = htons((uint16_t)ay_raw);
        frame.accel_raw_be[2] = htons((uint16_t)az_raw);
        frame.temp_mdegc_be = htonl((uint32_t)temp_mdegc);
        frame.gyro_mdps_be[0] = htonl((uint32_t)gx_mdps);
        frame.gyro_mdps_be[1] = htonl((uint32_t)gy_mdps);
        frame.gyro_mdps_be[2] = htonl((uint32_t)gz_mdps);
        frame.accel_ums2_be[0] = htonl((uint32_t)ax_ums2);
        frame.accel_ums2_be[1] = htonl((uint32_t)ay_ums2);
        frame.accel_ums2_be[2] = htonl((uint32_t)az_ums2);

        ssize_t sent = sendto(udp_fd,
                              &frame,
                              sizeof(frame),
                              0,
                              (struct sockaddr *)&dst_addr,
                              sizeof(dst_addr));
        if (sent != (ssize_t)sizeof(frame)) {
            perror("sendto UDP");
            break;
        }

        nanosleep(&period, NULL);
    }

    close(udp_fd);
    close(i2c_fd);
    return 0;
}
