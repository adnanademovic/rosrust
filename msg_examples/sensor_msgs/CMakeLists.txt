cmake_minimum_required(VERSION 3.0.2)
project(sensor_msgs)

find_package(catkin REQUIRED COMPONENTS geometry_msgs message_generation std_msgs)

# For point_cloud2.py
catkin_python_setup()

include_directories(include)

add_message_files(
  DIRECTORY msg
  FILES
  BatteryState.msg
  CameraInfo.msg
  ChannelFloat32.msg
  CompressedImage.msg
  FluidPressure.msg
  Illuminance.msg
  Image.msg
  Imu.msg
  JointState.msg
  Joy.msg
  JoyFeedback.msg
  JoyFeedbackArray.msg
  LaserEcho.msg
  LaserScan.msg
  MagneticField.msg
  MultiDOFJointState.msg
  MultiEchoLaserScan.msg
  NavSatFix.msg
  NavSatStatus.msg
  PointCloud.msg
  PointCloud2.msg
  PointField.msg
  Range.msg
  RegionOfInterest.msg
  RelativeHumidity.msg
  Temperature.msg
  TimeReference.msg)

add_service_files(
  DIRECTORY srv
  FILES SetCameraInfo.srv)

generate_messages(DEPENDENCIES geometry_msgs std_msgs)

catkin_package(
  INCLUDE_DIRS include
  CATKIN_DEPENDS geometry_msgs message_runtime std_msgs)

install(DIRECTORY include/${PROJECT_NAME}/
  DESTINATION ${CATKIN_PACKAGE_INCLUDE_DESTINATION}
  FILES_MATCHING PATTERN "*.h")

install(DIRECTORY migration_rules
  DESTINATION ${CATKIN_PACKAGE_SHARE_DESTINATION})

if (CATKIN_ENABLE_TESTING)
  add_subdirectory(test)
endif()
