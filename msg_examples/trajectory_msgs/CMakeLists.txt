cmake_minimum_required(VERSION 3.0.2)
project(trajectory_msgs)

find_package(catkin REQUIRED COMPONENTS message_generation std_msgs geometry_msgs)

add_message_files(
  DIRECTORY msg
  FILES
  JointTrajectory.msg
  JointTrajectoryPoint.msg
  MultiDOFJointTrajectory.msg
  MultiDOFJointTrajectoryPoint.msg
  )

generate_messages(DEPENDENCIES std_msgs geometry_msgs)

catkin_package(CATKIN_DEPENDS message_runtime std_msgs geometry_msgs)

install(DIRECTORY rules
  DESTINATION ${CATKIN_PACKAGE_SHARE_DESTINATION})
