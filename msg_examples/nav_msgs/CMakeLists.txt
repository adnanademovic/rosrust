cmake_minimum_required(VERSION 3.0.2)
project(nav_msgs)

find_package(catkin REQUIRED COMPONENTS geometry_msgs message_generation std_msgs actionlib_msgs)

add_message_files(
  DIRECTORY msg
  FILES
  GridCells.msg
  MapMetaData.msg
  OccupancyGrid.msg
  Odometry.msg
  Path.msg)

add_service_files(
  DIRECTORY srv
  FILES
  GetMap.srv
  GetPlan.srv
  SetMap.srv
  LoadMap.srv)

add_action_files(
  FILES
  GetMap.action)

generate_messages(DEPENDENCIES geometry_msgs std_msgs actionlib_msgs)

catkin_package(CATKIN_DEPENDS geometry_msgs message_runtime std_msgs actionlib_msgs)
