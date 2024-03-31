#include <SimpleFOC.h>
#include <Arduino.h>
#include "SimpleCAN.h"   // <- this is the only include required, it should be smart enough to find the correct subclass

//Motor and driver instance
BLDCMotor motor = BLDCMotor(6); 
BLDCDriver6PWM driver = BLDCDriver6PWM(A_PHASE_UH, A_PHASE_UL, A_PHASE_VH, A_PHASE_VL, A_PHASE_WH, A_PHASE_WL);
LowsideCurrentSense currentSense = LowsideCurrentSense(0.003f, -64.0f/7.0f, A_OP1_OUT, A_OP2_OUT, A_OP3_OUT);
MagneticSensorI2C sensor = MagneticSensorI2C(AS5600_I2C);

// Commander instance
Commander command = Commander(Serial);

float target_velocity = 0;
void doTarget(char* cmd) { command.scalar(&target_velocity, cmd); }

typedef struct velocityMessage {
  int16_t m1_velocity_rpm;
  int16_t m2_velocity_rpm;
  int16_t m3_velocity_rpm;
  int16_t m4_velocity_rpm;
} velocityMessage;

void setupCurrentSense() {
    currentSense.linkDriver(&driver);
    // current sensing
    currentSense.init();
    // no need for aligning
    currentSense.skip_align = true;
    motor.linkCurrentSense(&currentSense);
}

void setupSensor() {
  // initialise magnetic sensor hardware
  sensor.init();
  // link the motor to the sensor
  motor.linkSensor(&sensor);
}

void setupDriver() {
    //Driver initialization
    driver.voltage_power_supply = 12.1;
    driver.init();

}

void setupMotor() {
  // aligning voltage [V]
  motor.voltage_sensor_align = 3;
  // index search velocity [rad/s]
  motor.velocity_index_search = 3;

  // velocity PI controller parameters
  motor.PID_velocity.P = 0.2;
  motor.PID_velocity.I = 10;
  motor.pole_pairs = 7;

  // motor.voltage_limit = 6;

  // default voltage_power_supply

  //Limiting motor movements
  motor.phase_resistance = 0.3; // [Ohm]
  motor.current_limit = 6;   // [Amps] - if phase resistance defined
  //motor.voltage_limit = 1;   // [V] - if phase resistance not defined
  //motor.velocity_limit = 5; // [rad/s] cca 50rpm
  //motor.voltage_limit = 1;

  // jerk control using voltage voltage ramp
  // default value is 300 volts per sec  ~ 0.3V per millisecond
  motor.PID_velocity.output_ramp = 1000;

  // velocity low pass filtering time constant
  motor.LPF_velocity.Tf = 0.01;
  //Control loop setup
  motor.controller = MotionControlType::velocity;

  motor.velocity_limit = 100;


  //Init motor
  motor.linkDriver(&driver);

  motor.useMonitoring(Serial);
  motor.init();

  // align encoder and start FOC
  motor.initFOC();

}


void setup() {

    // Start serial communication
    Serial.begin(115200);

    // Configure CAN Bus

    CAN.logTo(&Serial);

    CAN.begin(125000);


    // configure i2C
    Wire.setClock(400000);
    
    setupDriver();

    setupCurrentSense();

    setupSensor();

    setupMotor();

    // add target command T
    command.add('T', doTarget, "target");
    command.add('t', doTarget, "target");

    Serial.println("Motor ready!");
    Serial.println("Set target velocity [rad/s]");

    delay(1000);
}

void loop() {

    //Motor loop, as quick as possible
  //   sensor.update();
  
  // // display the angle and the angular velocity to the terminal
  //   Serial.print(sensor.getAngle());
  //   Serial.print("\t");
  //   Serial.println(sensor.getVelocity());

  // Check if we got a CAN message
  if (CAN.available() > 0)
  {
      CanMsg const rxMsg = CAN.read();

      Serial.print("polling read: ");
      if (rxMsg.isExtendedId())
      {
          Serial.print(rxMsg.getExtendedId(), HEX);
          Serial.println(" Extended ✅");
      }
      else
      {
          Serial.print(rxMsg.getStandardId(), HEX);
          Serial.println(" Standard ✅");
          
      }
      Serial.println("I got this message: ");

      for (uint8_t i = 0; i < rxMsg.data_length; i++)
      {
          Serial.print(rxMsg.data[i], HEX);
          Serial.print(" ");
      }
  }
    motor.loopFOC();
    motor.move(target_velocity);


    //User communication
    command.run();

    //Monitoring
    // motor.monitor();
}