cmake_minimum_required(VERSION 3.0.2)
project(actionlib_msgs)

find_package(catkin REQUIRED COMPONENTS message_generation std_msgs)

add_message_files(
  DIRECTORY msg
  FILES
  GoalID.msg
  GoalStatus.msg
  GoalStatusArray.msg)
generate_messages(DEPENDENCIES std_msgs)

catkin_package(
  CATKIN_DEPENDS message_runtime std_msgs
  CFG_EXTRAS actionlib_msgs-extras.cmake)

# install the .action -> .msg generator
catkin_install_python(PROGRAMS scripts/genaction.py
  DESTINATION ${CATKIN_PACKAGE_BIN_DESTINATION})

# install the legacy rosbuild cmake support
install(FILES cmake/actionbuild.cmake
  DESTINATION ${CATKIN_PACKAGE_SHARE_DESTINATION}/cmake)
