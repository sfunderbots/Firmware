#!/bin/bash

# Check if the correct number of arguments are provided
if [ "$#" -ne 4 ]; then
    echo "Usage: $0 <velocity1> <velocity2> <velocity3> <velocity4>"
    exit 1
fi

# Function to convert decimal fixed-point value to integer
# Multiply by 100 to shift two decimal places and remove decimal point
to_integer() {
    printf "%.0f" "$(echo "$1 * 100" | bc)"
}

# Function to convert integer to little endian hexadecimal representation
to_little_endian_hex() {
    printf "%04x" "$((($1 & 0xFF) << 8 | ($1 >> 8) & 0xFF))"
}

# Extract velocities from arguments, convert to integers, and convert to little endian hexadecimal representation
velocity1_int=$(to_integer $1)
velocity2_int=$(to_integer $2)
velocity3_int=$(to_integer $3)
velocity4_int=$(to_integer $4)

velocity1_hex=$(to_little_endian_hex $velocity1_int)
velocity2_hex=$(to_little_endian_hex $velocity2_int)
velocity3_hex=$(to_little_endian_hex $velocity3_int)
velocity4_hex=$(to_little_endian_hex $velocity4_int)

# Concatenate velocities to form CAN data
can_data="${velocity1_hex}${velocity2_hex}${velocity3_hex}${velocity4_hex}"

# Send CAN frame using cansend
cansend can0 123#${can_data}
