cmake_minimum_required(VERSION 3.0.2)
project(stereo_msgs)

find_package(catkin REQUIRED COMPONENTS sensor_msgs message_generation std_msgs)

add_message_files(
  DIRECTORY msg
  FILES
  DisparityImage.msg)

generate_messages(DEPENDENCIES sensor_msgs std_msgs)

catkin_package(CATKIN_DEPENDS message_runtime sensor_msgs std_msgs)
