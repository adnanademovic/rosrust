cmake_minimum_required(VERSION 3.0.2)
project(diagnostic_msgs)

find_package(catkin REQUIRED COMPONENTS message_generation std_msgs)

add_message_files(
  DIRECTORY msg
  FILES
  DiagnosticArray.msg
  DiagnosticStatus.msg
  KeyValue.msg)

add_service_files(
  DIRECTORY srv
  FILES
  AddDiagnostics.srv
  SelfTest.srv)

generate_messages(DEPENDENCIES std_msgs)

catkin_package(CATKIN_DEPENDS message_runtime std_msgs)
